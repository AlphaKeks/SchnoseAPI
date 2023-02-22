use {
	super::{Record, RecordQuery},
	crate::{
		models::{Response, ResponseBody},
		routes::maps::Course,
		Error, GlobalState,
	},
	axum::{
		extract::{Query, State},
		Json,
	},
	chrono::NaiveDateTime,
	database::{
		crd::read::{get_map, get_player},
		schemas::{account_id_to_steam_id64, FancyPlayer},
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
	map: Option<String>,
	player: Option<String>,
	has_teleports: Option<bool>,
	created_after: Option<String>,
	created_before: Option<String>,
	limit: Option<u32>,
}

pub(crate) async fn get(
	Query(params): Query<Params>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Vec<Record>> {
	let start = Instant::now();
	debug!("[records::player::get]");
	debug!("> `params`: {params:#?}");

	let mut inner_query = String::from("SELECT r_inner.* FROM records AS r_inner");

	// If there are no parameters specified, we can `LIMIT` the query earlier, resulting in _much_
	// faster results.
	let mut limit_early = true;

	if let Some(mode) = params.mode {
		let mode_id = mode.parse::<Mode>()? as u8;
		inner_query.push_str(&format!(
			"\n  JOIN modes AS mode ON mode.id = r_inner.mode_id AND mode.id = {mode_id}"
		));
		limit_early = false;
	}

	match (params.stage, params.map) {
		(Some(stage), None) => {
			inner_query.push_str(&format!(
				"\n  JOIN courses AS c ON c.id = r_inner.course_id AND c.stage = {stage}"
			));
			limit_early = false;
		}
		(None, Some(map_ident)) => {
			let map_ident = map_ident.parse::<MapIdentifier>()?;
			let map_id = get_map(map_ident, &pool)
				.await
				.map(|map| map.id)?;
			inner_query.push_str(&format!(
				"\n  JOIN courses AS c ON c.id = r_inner.course_id AND c.map_id = {map_id}"
			));
			limit_early = false;
		}
		(Some(stage), Some(map_ident)) => {
			let map_ident = map_ident.parse::<MapIdentifier>()?;
			let map_id = get_map(map_ident, &pool)
				.await
				.map(|map| map.id)?;
			inner_query.push_str(&format!(
				"\n  JOIN courses AS c ON c.id = r_inner.course_id AND c.stage = {stage} AND c.map_id = {map_id}"
			));
			limit_early = false;
		}
		(None, None) => {}
	};

	// let player_id = get_player(player_ident, &pool)
	// 	.await
	// 	.map(|player_row| player_row.id)?;
	//
	// inner_query.push_str(&format!(
	// 	"\n  JOIN players AS p ON p.id = r_inner.player_id AND p.id = {player_id}"
	// ));

	if let Some(player_ident) = params.player {
		let player_ident = player_ident.parse::<PlayerIdentifier>()?;
		let player_id = get_player(player_ident, &pool)
			.await
			.map(|player_row| player_row.id)?;

		inner_query.push_str(&format!(
			"\n  JOIN players AS p ON p.id = r_inner.player_id AND p.id = {player_id}"
		));
		limit_early = false;
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
			limit_early = false;
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
			limit_early = false;
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
			limit_early = false;
		}
		_ => {}
	};

	if let Some(has_teleports) = params.has_teleports {
		inner_filter.push_str(&format!(
			"\n  AND r_inner.teleports {} 0",
			if has_teleports { ">" } else { "=" }
		));
		limit_early = false;
	}

	let limit = params
		.limit
		.map_or(100, |limit| limit.min(250));

	inner_query.push_str(&inner_filter.replacen("AND", "WHERE", 1));

	let query = if limit_early {
		format!(
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
			  SELECT * FROM records AS r_inner
			  ORDER BY r_inner.created_on DESC
			  LIMIT {limit}
			) AS r
			JOIN courses AS c ON c.id = r.course_id
			JOIN maps AS map ON map.id = c.map_id
			JOIN modes AS mode ON mode.id = r.mode_id
			JOIN players AS p ON p.id = r.player_id
			JOIN servers AS s ON s.id = r.server_id
			ORDER BY r.created_on DESC
			"#,
		)
	} else {
		format!(
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
			  {inner_query}
			) AS r
			JOIN courses AS c ON c.id = r.course_id
			JOIN maps AS map ON map.id = c.map_id
			JOIN modes AS mode ON mode.id = r.mode_id
			JOIN players AS p ON p.id = r.player_id
			JOIN servers AS s ON s.id = r.server_id
			ORDER BY r.created_on DESC
			LIMIT {limit}
			"#,
		)
	};

	let mut result = Vec::new();
	for record_query in sqlx::query_as::<_, RecordQuery>(&query)
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
