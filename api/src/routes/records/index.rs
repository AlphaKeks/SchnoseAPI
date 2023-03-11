use {
	super::{Record, RecordQuery},
	crate::{routes::maps::Course, Error, GlobalState, Response, ResponseBody},
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
	sqlx::QueryBuilder,
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

	let mut query = QueryBuilder::new(
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
		  SELECT r_inner.* FROM records AS r_inner
		"#,
	);

	let limit = params.limit.unwrap_or(100);

	let no_params = params.mode.is_none()
		&& params.stage.is_none()
		&& params.map.is_none()
		&& params.player.is_none()
		&& params.has_teleports.is_none()
		&& params.created_after.is_none()
		&& params.created_before.is_none()
		&& params.limit.is_none();

	if no_params {
		query
			.push(" ORDER BY r_inner.created_on DESC")
			.push(" LIMIT ")
			.push_bind(limit)
			.push(") AS r ");
	} else {
		if let Some(mode) = params.mode {
			let mode_id = mode.parse::<Mode>()? as u8;
			query
				.push(" JOIN modes AS mode ON mode.id = r_inner.mode_id AND mode.id = ")
				.push_bind(mode_id);
		}

		match (params.stage, params.map) {
			(Some(stage), None) => {
				query
					.push(" JOIN courses AS c ON c.id = r_inner.course_id AND c.stage = ")
					.push_bind(stage);
			}
			(None, Some(map_ident)) => {
				let map_ident = map_ident.parse::<MapIdentifier>()?;
				let map_id = get_map(map_ident, &pool)
					.await
					.map(|map| map.id)?;

				query
					.push(" JOIN courses AS c ON c.id = r_inner.course_id AND c.map_id = ")
					.push_bind(map_id);
			}
			(Some(stage), Some(map_ident)) => {
				let map_ident = map_ident.parse::<MapIdentifier>()?;
				let map_id = get_map(map_ident, &pool)
					.await
					.map(|map| map.id)?;

				query
					.push(" JOIN courses AS c ON c.id = r_inner.course_id AND c.stage = ")
					.push_bind(stage)
					.push(" AND c.map_id = ")
					.push_bind(map_id);
			}
			(None, None) => {}
		};

		if let Some(player_ident) = params.player {
			let player_ident = player_ident.parse::<PlayerIdentifier>()?;
			let player_id = get_player(player_ident, &pool)
				.await
				.map(|player_row| player_row.id)?;

			query
				.push(" JOIN players AS p ON p.id = r_inner.player_id AND p.id = ")
				.push_bind(player_id);
		}

		let mut multiple_filters = false;

		match (params.created_after, params.created_before) {
			(Some(created_after), None) => {
				let created_after =
					NaiveDateTime::parse_from_str(&created_after, "%Y-%m-%dT%H:%M:%S")?
						.format("%Y-%m-%d %H:%M:%S")
						.to_string();

				query
					.push(" WHERE r_inner.created_on > ")
					.push_bind(created_after);

				multiple_filters = true;
			}
			(None, Some(created_before)) => {
				let created_before =
					NaiveDateTime::parse_from_str(&created_before, "%Y-%m-%dT%H:%M:%S")?
						.format("%Y-%m-%d %H:%M:%S")
						.to_string();

				query
					.push(" WHERE r_inner.created_on < ")
					.push_bind(created_before);

				multiple_filters = true;
			}
			(Some(created_after), Some(created_before)) => {
				let created_after =
					NaiveDateTime::parse_from_str(&created_after, "%Y-%m-%dT%H:%M:%S")?;
				let created_before =
					NaiveDateTime::parse_from_str(&created_before, "%Y-%m-%dT%H:%M:%S")?;

				if created_after.timestamp() > created_before.timestamp() {
					return Err(Error::DateRange);
				}

				query
					.push(" WHERE r_inner.created_on > ")
					.push_bind(created_after.to_string())
					.push(" AND r_inner.created_on < ")
					.push_bind(created_before.to_string());

				multiple_filters = true;
			}
			_ => {}
		};

		if let Some(has_teleports) = params.has_teleports {
			query
				.push(if multiple_filters { " AND " } else { " WHERE " })
				.push(format!(" r_inner.teleports {} 0", if has_teleports { ">" } else { "=" }));
		}

		query
			.push(" ORDER BY r_inner.created_on DESC")
			.push(" LIMIT ")
			.push_bind(limit)
			.push(") AS r ");
	}

	query.push(
		r#"
		JOIN courses AS c ON c.id = r.course_id
		JOIN maps AS map ON map.id = c.map_id
		JOIN modes AS mode ON mode.id = r.mode_id
		JOIN players AS p ON p.id = r.player_id
		JOIN servers AS s ON s.id = r.server_id
		ORDER BY r.created_on DESC
		"#,
	);

	let query_result = query
		.build_query_as::<RecordQuery>()
		.fetch_all(&pool)
		.await?;

	if query_result.is_empty() {
		return Err(sqlx::Error::RowNotFound.into());
	}

	let mut result = Vec::new();
	for record_query in query_result {
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
			created_on: record_query.created_on,
		});
	}

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
