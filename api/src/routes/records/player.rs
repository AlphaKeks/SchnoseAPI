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
	has_teleports: Option<bool>,
	created_after: Option<String>,
	created_before: Option<String>,
	limit: Option<u32>,
}

pub(crate) async fn get(
	Path(player_ident): Path<String>,
	Query(params): Query<Params>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Vec<Record>> {
	let start = Instant::now();
	debug!("[records::player::get]");
	debug!("> `player_ident`: {player_ident:#?}");
	let player_ident = player_ident.parse::<PlayerIdentifier>()?;
	debug!("> `player_ident`: {player_ident:#?}");
	debug!("> `params`: {params:#?}");

	let player_id = get_player(player_ident, &pool)
		.await
		.map(|player_row| player_row.id)?;

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
			  SELECT
			    r_inner.course_id,
			    MIN(r_inner.time) AS time
			  FROM records AS r_inner
			  JOIN players AS p ON p.id = r_inner.player_id AND r_inner.player_id =
		"#,
	);

	query.push_bind(player_id);

	if let Some(stage) = params.stage {
		query
			.push(" JOIN courses AS c ON c.id = r_inner.course_id AND c.stage = ")
			.push_bind(stage);
	}

	if let Some(mode) = &params.mode {
		let mode = mode.parse::<Mode>()?;
		query
			.push(" JOIN modes AS mode ON mode.id = r_inner.mode_id AND mode.id = ")
			.push_bind(mode as u8);
	}

	if let Some(map_ident) = &params.map {
		let map_ident = map_ident.parse::<MapIdentifier>()?;
		let map_id = if let MapIdentifier::ID(map_id) = map_ident {
			map_id as u16
		} else {
			get_map(map_ident, &pool)
				.await
				.map(|map_row| map_row.id)?
		};

		query
			.push(" JOIN maps AS m ON m.id = ")
			.push_bind(map_id);
	}

	let mut multiple_filters = false;

	match (&params.created_after, &params.created_before) {
		(Some(created_after), None) => {
			let created_after = NaiveDateTime::parse_from_str(created_after, "%Y-%m-%dT%H:%M:%S")?
				.format("%Y-%m-%d %H:%M:%S")
				.to_string();

			query
				.push(" WHERE r_inner.created_on > ")
				.push_bind(created_after);

			multiple_filters = true;
		}
		(None, Some(created_before)) => {
			let created_before =
				NaiveDateTime::parse_from_str(created_before, "%Y-%m-%dT%H:%M:%S")?
					.format("%Y-%m-%d %H:%M:%S")
					.to_string();

			query
				.push(" WHERE r_inner.created_on < ")
				.push_bind(created_before);

			multiple_filters = true;
		}
		(Some(created_after), Some(created_before)) => {
			let created_after = NaiveDateTime::parse_from_str(created_after, "%Y-%m-%dT%H:%M:%S")?;
			let created_before =
				NaiveDateTime::parse_from_str(created_before, "%Y-%m-%dT%H:%M:%S")?;

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
		.push(
			r#"
		    GROUP BY r_inner.course_id
			) AS pb
			JOIN records AS r
			  ON r.course_id = pb.course_id
			  AND r.time = pb.time
			JOIN courses AS c ON c.id = r.course_id
			JOIN maps AS map ON map.id = c.map_id
			JOIN modes AS mode ON mode.id = r.mode_id
			JOIN players AS p ON p.id = r.player_id AND r.player_id =
			"#,
		)
		.push_bind(player_id)
		.push(" JOIN servers AS s ON s.id = r.server_id ");

	multiple_filters = false;

	if let Some(stage) = params.stage {
		query
			.push(" WHERE c.id = r.course_id AND c.stage = ")
			.push_bind(stage);
		multiple_filters = true;
	}

	if let Some(mode) = params.mode {
		let mode_id = mode.parse::<Mode>()? as u8;
		query
			.push(if multiple_filters { " AND " } else { " WHERE " })
			.push(" mode.id = r.mode_id AND mode.id = ")
			.push_bind(mode_id);
		multiple_filters = true;
	}

	if let Some(map_ident) = params.map {
		let map_ident = map_ident.parse::<MapIdentifier>()?;
		let map_id = if let MapIdentifier::ID(map_id) = map_ident {
			map_id as u16
		} else {
			get_map(map_ident, &pool)
				.await
				.map(|map_row| map_row.id)?
		};

		query
			.push(if multiple_filters { " AND " } else { " WHERE " })
			.push(" map.id = ")
			.push_bind(map_id);
	}

	match (params.created_after, params.created_before) {
		(Some(created_after), None) => {
			let created_after = NaiveDateTime::parse_from_str(&created_after, "%Y-%m-%dT%H:%M:%S")?
				.format("%Y-%m-%d %H:%M:%S")
				.to_string();

			query
				.push(if multiple_filters { " AND " } else { " WHERE " })
				.push(" r.created_on > ")
				.push_bind(created_after);

			multiple_filters = true;
		}
		(None, Some(created_before)) => {
			let created_before =
				NaiveDateTime::parse_from_str(&created_before, "%Y-%m-%dT%H:%M:%S")?
					.format("%Y-%m-%d %H:%M:%S")
					.to_string();

			query
				.push(if multiple_filters { " AND " } else { " WHERE " })
				.push(" r.created_on < ")
				.push_bind(created_before);

			multiple_filters = true;
		}
		(Some(created_after), Some(created_before)) => {
			let created_after = NaiveDateTime::parse_from_str(&created_after, "%Y-%m-%dT%H:%M:%S")?;
			let created_before =
				NaiveDateTime::parse_from_str(&created_before, "%Y-%m-%dT%H:%M:%S")?;

			if created_after.timestamp() > created_before.timestamp() {
				return Err(Error::DateRange);
			}

			query
				.push(if multiple_filters { " AND " } else { " WHERE " })
				.push(" r.created_on > ")
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
			.push(format!(" r.teleports {} 0", if has_teleports { ">" } else { "=" }));
	}

	query
		.push(
			r#"
			ORDER BY r.created_on DESC, c.stage ASC
			LIMIT
			"#,
		)
		.push_bind(params.limit.unwrap_or(100));

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
