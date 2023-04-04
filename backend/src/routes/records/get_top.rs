use {
	crate::{Error, GlobalState},
	axum::{
		extract::{Query, State},
		Json,
	},
	backend::{
		models::records::{RecordParams, RecordResponse, RecordRow},
		Response,
	},
	sqlx::QueryBuilder,
	tracing::debug,
};

pub async fn get_top(
	Query(params): Query<RecordParams>,
	State(global_state): State<GlobalState>,
) -> Response<Vec<RecordResponse>> {
	debug!("[records::get_index]");
	debug!("> `params`: {params:#?}");

	let mut query = QueryBuilder::new(
		r#"
		SELECT
		  record.id AS id,
		  player.name AS player_name,
		  player.id AS player_id,
		  course.map_id AS map_id,
		  map.name AS map_name,
		  course.stage AS stage,
		  record.mode_id AS mode_id,
		  record.time AS time,
		  record.teleports AS teleports,
		  record.created_on AS created_on
		FROM (
		  SELECT
		    record.id,
		    MIN(record.time) AS "time"
		  FROM records AS record
		  JOIN players AS player ON player.id = record.player_id
		  JOIN courses AS course ON course.id = record.course_id
		  JOIN maps AS map ON map.id = course.map_id
		"#,
	);

	// This gets changed to `" AND "` after any filter has been applied.
	let mut clause = " WHERE ";

	if let Some(map_identifier) = params.map {
		match map_identifier {
			gokz_rs::MapIdentifier::ID(map_id) => {
				query
					.push(r#" WHERE map.id = "#)
					.push_bind(map_id);
			}
			gokz_rs::MapIdentifier::Name(map_name) => {
				query
					.push(r#" WHERE map.name LIKE "#)
					.push_bind(format!(r#"%{map_name}%"#));
			}
		};

		clause = " AND ";
	}

	if let Some(stage) = params.stage {
		query
			.push(clause)
			.push(r#" course.stage = "#)
			.push_bind(stage);
		clause = " AND ";
	}

	if let Some(mode) = params.mode {
		query
			.push(clause)
			.push(r#" record.mode_id = "#)
			.push_bind(mode as u8);
		clause = " AND ";
	}

	if let Some(player_identifier) = params.player {
		query.push(clause);

		match player_identifier {
			gokz_rs::PlayerIdentifier::SteamID(steam_id) => {
				query
					.push(r#" player.id = "#)
					.push_bind(steam_id.as_id32());
			}
			gokz_rs::PlayerIdentifier::Name(player_name) => {
				query
					.push(r#" player.name LIKE "#)
					.push_bind(format!(r#"%{player_name}%"#));
			}
		};

		clause = " AND ";
	}

	if !matches!(params.allow_bans, Some(true)) {
		query
			.push(clause)
			.push(r#" player.is_banned = 0 "#);

		clause = " AND ";
	}

	if let Some(has_teleports) = params.has_teleports {
		query
			.push(clause)
			.push(r#" record.teleports "#)
			.push(match has_teleports {
				true => ">",
				false => "=",
			})
			.push(" 0 ");

		clause = " AND ";
	}

	match (params.created_after, params.created_before) {
		(None, None) => {}
		(Some(created_after), None) => {
			query
				.push(clause)
				.push(" record.created_on > ")
				.push_bind(created_after);
		}
		(None, Some(created_before)) => {
			query
				.push(clause)
				.push(" record.created_on < ")
				.push_bind(created_before);
		}
		(Some(created_after), Some(created_before)) => {
			if created_before <= created_after {
				return Err(Error::InvalidDateBounds);
			}

			query
				.push(clause)
				.push(" record.created_on > ")
				.push_bind(created_after)
				.push(" AND ")
				.push(" record.created_on < ")
				.push_bind(created_before);
		}
	};

	query.push(
		r#"
		  GROUP BY record.course_id, record.player_id, record.mode_id
		) AS top_times
		JOIN records AS record
		  ON record.id = top_times.id
		JOIN players AS player ON player.id = record.player_id
		JOIN courses AS course ON course.id = record.course_id
		JOIN maps AS map ON map.id = course.map_id
		"#,
	);

	query
		.push(r#" ORDER BY record.time "#)
		.push(r#" LIMIT "#)
		.push_bind(params.limit.unwrap_or(100).min(500));

	let result = query
		.build_query_as::<RecordRow>()
		.fetch_all(&global_state.conn)
		.await?
		.into_iter()
		.flat_map(TryInto::try_into)
		.collect();

	debug!("Database result: {result:#?}");

	Ok(Json(result))
}
