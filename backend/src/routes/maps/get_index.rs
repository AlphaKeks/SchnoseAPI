use {
	crate::{DatabaseError, Error, GlobalState},
	axum::{
		extract::{Query, State},
		Json,
	},
	backend::{
		models::maps::{MapParams, MapResponse, MapRow},
		Response,
	},
	gokz_rs::PlayerIdentifier,
	sqlx::QueryBuilder,
	tracing::debug,
};

pub async fn get_index(
	Query(params): Query<MapParams>,
	State(global_state): State<GlobalState>,
) -> Response<Vec<MapResponse>> {
	debug!("[maps::get_index]");
	debug!("> {params:#?}");

	let mut query =
		QueryBuilder::new("SELECT map.*, courses, mapper.*, approver.* FROM maps AS map");

	// This gets changed to `" AND "` after any filter has been applied.
	let mut clause = " WHERE ";

	if let Some(name) = params.name {
		query
			.push(r#" WHERE map.name LIKE "#)
			.push_bind(format!(r#"%{name}%"#));
		clause = " AND ";
	}

	if let Some(stages) = params.stages {
		query
			.push(clause)
			.push(" map.stages = ")
			.push_bind(u8::from(stages));
		clause = " AND ";
	}

	if let Some(validated) = params.validated {
		query
			.push(clause)
			.push(" map.validated = ")
			.push_bind(validated as u8);
		clause = " AND ";
	}

	match (params.created_after, params.created_before) {
		(None, None) => {}
		(Some(created_after), None) => {
			query
				.push(clause)
				.push(" map.created_on > ")
				.push_bind(created_after);
		}
		(None, Some(created_before)) => {
			query
				.push(clause)
				.push(" map.created_on < ")
				.push_bind(created_before);
		}
		(Some(created_after), Some(created_before)) => {
			if created_before <= created_after {
				todo!();
			}

			query
				.push(clause)
				.push(" map.created_on > ")
				.push_bind(created_after)
				.push(" AND ")
				.push(" map.created_on < ")
				.push_bind(created_before);
		}
	};

	if let Some(mapper) = params.mapper {
		query.push(" JOIN players AS mapper ON ");
		match mapper {
			PlayerIdentifier::SteamID(steam_id) => {
				query
					.push(" mapper.id = ")
					.push_bind(steam_id.as_id32());
			}
			PlayerIdentifier::Name(name) => {
				query
					.push(" mapper.name LIKE ")
					.push_bind(format!(r#"%{name}%"#));
			}
		};
	}

	if let Some(approver) = params.approver {
		query.push(" JOIN players AS approver ON ");
		match approver {
			PlayerIdentifier::SteamID(steam_id) => {
				query
					.push(" approver.id = ")
					.push_bind(steam_id.as_id32());
			}
			PlayerIdentifier::Name(name) => {
				query
					.push(" approver.name LIKE ")
					.push_bind(format!(r#"%{name}%"#));
			}
		};
	}

	query.push("JOIN courses AS courses ON courses.map_id = map.id");

	if let Some(tier) = params.tier {
		query
			.push(" AND courses.kzt = ")
			.push_bind(tier as u8);
	}

	query
		.push(" LIMIT ")
		.push_bind(params.limit.unwrap_or(500).min(500));

	let result: Vec<_> = query
		.build_query_as::<MapRow>()
		.fetch_all(&global_state.conn)
		.await?
		.into_iter()
		.filter_map(|row| {
			let parsed = row.try_into();
			debug!("Trying to parse row: {parsed:#?}");
			parsed.ok()
		})
		.collect();

	debug!("Database result: {result:#?}");

	if result.is_empty() {
		return Err(Error::Database {
			kind: DatabaseError::NoRows,
		});
	}

	Ok(Json(result))
}
