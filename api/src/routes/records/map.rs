use {
	super::{Record, RecordQuery},
	crate::{routes::maps::Course, Error, GlobalState, Response, ResponseBody},
	axum::{
		extract::{Path, Query, State},
		Json,
	},
	chrono::NaiveDateTime,
	database::{
		crd::read::{get_map, get_player},
		schemas::{
			account_id_to_steam_id64, steam_id64_to_account_id, steam_id_to_account_id, FancyPlayer,
		},
	},
	gokz_rs::prelude::*,
	log::debug,
	serde::Deserialize,
	std::time::Instant,
};

#[derive(Debug, Deserialize)]
pub(crate) struct Params {
	mode: Option<String>,
	stage: Option<u8>,
	player: Option<String>,
	has_teleports: Option<bool>,
	created_after: Option<String>,
	created_before: Option<String>,
	limit: Option<u32>,
}

pub(crate) async fn get(
	Path(map_ident): Path<String>,
	Query(params): Query<Params>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Vec<Record>> {
	let start = Instant::now();
	debug!("[records::player::get]");
	debug!("> `map_ident`: {map_ident:#?}");
	let map_ident = map_ident.parse::<MapIdentifier>()?;
	debug!("> `map_ident`: {map_ident:#?}");
	debug!("> `params`: {params:#?}");

	if let MapIdentifier::Name(map_name) = &map_ident {
		if map_name.contains('&') {
			return Err(Error::Input {
				message: format!(
					"Interpreted `{map_name}` as a map name. You probably meant to use a `?` instead of the first `&`."
				),
				expected: String::from("?` instead of `&"),
			});
		}
	}

	let map_id = get_map(map_ident, &pool)
		.await
		.map(|map_row| map_row.id)?;

	let mut inner_query = String::new();

	inner_query.push_str(&format!(
		"\n  JOIN courses AS c ON c.id = r_inner.course_id AND c.map_id = {map_id} {}",
		if let Some(stage) = params.stage {
			format!("AND c.stage = {stage}")
		} else {
			String::new()
		}
	));

	if let Some(mode) = params.mode {
		let mode_id = mode.parse::<Mode>()? as u8;
		inner_query.push_str(&format!(
			"\n  JOIN modes AS mode ON mode.id = r_inner.mode_id AND mode.id = {mode_id}"
		));
	}

	if let Some(player_ident) = params.player {
		let player_id = match player_ident.parse::<PlayerIdentifier>()? {
			PlayerIdentifier::SteamID(steam_id) => steam_id_to_account_id(&steam_id.to_string())
				.ok_or(Error::Input {
					message: format!("Interpreted `{steam_id}` as a SteamID but it was invalid."),
					expected: String::from("a valid SteamID"),
				})?,
			PlayerIdentifier::SteamID64(steam_id64) => steam_id64_to_account_id(steam_id64)?,
			player_ident => get_player(player_ident, &pool)
				.await
				.map(|player_row| player_row.id)?,
		};

		inner_query.push_str(&format!("\n  JOIN players AS p ON p.id = {player_id}"));
	}

	let mut inner_filter = String::new();

	match (params.created_after, params.created_before) {
		(Some(created_after), None) => {
			let created_after = NaiveDateTime::parse_from_str(&created_after, "%Y-%m-%dT%H:%M:%S")?
				.format("%Y-%m-%d %H:%M:%S");

			inner_filter.push_str(&format!(
				r#"
				  AND r_inner.created_on > "{created_after}"
				"#
			));
		}
		(None, Some(created_before)) => {
			let created_before =
				NaiveDateTime::parse_from_str(&created_before, "%Y-%m-%dT%H:%M:%S")?
					.format("%Y-%m-%d %H:%M:%S");

			inner_filter.push_str(&format!(
				r#"
				  AND r_inner.created_on < "{created_before}"
				"#
			));
		}
		(Some(created_after), Some(created_before)) => {
			let created_after = NaiveDateTime::parse_from_str(&created_after, "%Y-%m-%dT%H:%M:%S")?;
			let created_before =
				NaiveDateTime::parse_from_str(&created_before, "%Y-%m-%dT%H:%M:%S")?;

			if created_after.timestamp() > created_before.timestamp() {
				return Err(Error::DateRange);
			}

			inner_filter.push_str(&format!(
				r#"
				  AND r_inner.created_on > "{}"
				  AND r_inner.created_on < "{}"
				"#,
				created_after.format("%Y-%m-%d %H:%M:%S"),
				created_before.format("%Y-%m-%d %H:%M:%S"),
			));
		}
		_ => {}
	};

	if let Some(has_teleports) = params.has_teleports {
		inner_filter.push_str(&format!(
			"\n  AND r_inner.teleports {} 0",
			if has_teleports { ">" } else { "=" }
		));
	}

	let limit = params
		.limit
		.map_or(100, |limit| limit.min(250));

	debug!("FILTER: {inner_filter}");

	let mut result = Vec::new();
	for record_query in sqlx::query_as::<_, RecordQuery>(&format!(
		r#"
			SELECT
			  r.id AS id,
			  map.id AS map_id,
			  map.name AS map_name,
			  c.id AS course_id,
			  c.stage AS stage,
			  c.kzt AS kzt,
			  c.kzt_difficulty AS kzt_difficulty,
			  c.skz AS skz,
			  c.skz_difficulty AS skz_difficulty,
			  c.vnl AS vnl,
			  c.vnl_difficulty AS vnl_difficulty,
			  mode.name AS mode,
			  p.id AS player_id,
			  p.name AS player_name,
			  p.is_banned AS player_is_banned,
			  s.name AS server_name,
			  r.time AS time,
			  r.teleports AS teleports,
			  r.created_on AS created_on
			FROM (
			  SELECT
			    r_inner.mode_id,
			    r_inner.course_id,
			    r_inner.player_id,
			    r_inner.teleports,
			    MIN(r_inner.time) AS time
			  FROM records AS r_inner
			  {inner_query}
			  {inner_filter}
			  GROUP BY r_inner.mode_id, r_inner.course_id, r_inner.player_id, r_inner.teleports
			) AS pb
			JOIN records AS r
			  ON r.mode_id = pb.mode_id
			  AND r.course_id = pb.course_id
			  AND r.player_id = pb.player_id
			  AND r.teleports = pb.teleports
			  AND r.time = pb.time
			JOIN courses AS c ON c.id = r.course_id
			JOIN maps AS map ON map.id = c.map_id
			JOIN modes AS mode ON mode.id = r.mode_id
			JOIN players AS p ON p.id = r.player_id
			JOIN servers AS s ON s.id = r.server_id
			ORDER BY c.stage ASC, r.time, r.created_on DESC
			LIMIT {limit}
		"#,
	))
	.fetch_all(&pool)
	.await?
	{
		let steam_id64 = account_id_to_steam_id64(record_query.player_id);
		let steam_id = SteamID::from(steam_id64);

		result.push(Record {
			id: record_query.id,
			map_name: record_query.map_name,
			course: Course {
				id: record_query.course_id,
				stage: record_query.stage,
				kzt: record_query.kzt,
				kzt_difficulty: record_query.kzt_difficulty,
				skz: record_query.skz,
				skz_difficulty: record_query.skz_difficulty,
				vnl: record_query.vnl,
				vnl_difficulty: record_query.vnl_difficulty,
			},
			mode: record_query.mode,
			player: FancyPlayer {
				id: record_query.player_id,
				name: record_query.player_name,
				steam_id: steam_id.to_string(),
				steam_id64: steam_id64.to_string(),
				is_banned: record_query.player_is_banned,
			},
			server_name: record_query.server_name,
			time: record_query.time,
			teleports: record_query.teleports,
			created_on: record_query.created_on.to_string(),
		});
	}

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
