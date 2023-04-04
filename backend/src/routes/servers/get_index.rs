use {
	crate::{DatabaseError, Error, GlobalState},
	axum::{
		extract::{Query, State},
		Json,
	},
	backend::{
		models::servers::{ServerParams, ServerResponse, ServerRow},
		Response,
	},
	gokz_rs::PlayerIdentifier,
	sqlx::QueryBuilder,
	tracing::debug,
};

pub async fn get_index(
	Query(params): Query<ServerParams>,
	State(global_state): State<GlobalState>,
) -> Response<Vec<ServerResponse>> {
	debug!("[servers::get_index]");
	debug!("> {params:#?}");

	let mut query = QueryBuilder::new(
		r#"
		SELECT
		  server.id AS id,
		  server.name AS name,
		  owner.id AS owner_id,
		  owner.name AS owner_name,
		  approver.id AS approver_id,
		  approver.name AS approver_name
		FROM servers AS server
		JOIN players AS owner ON owner.id = server.owned_by
		JOIN players AS approver ON approver.id = server.approved_by
		"#,
	);

	// This gets changed to `" AND "` after any filter has been applied.
	let mut clause = " WHERE ";

	if let Some(name) = params.name {
		query
			.push(r#" WHERE server.name LIKE "#)
			.push_bind(format!(r#"%{name}%"#));
		clause = " AND ";
	}

	if let Some(owner) = params.owned_by {
		query.push(clause);
		match owner {
			PlayerIdentifier::SteamID(steam_id) => {
				query
					.push(" owner.id = ")
					.push_bind(steam_id.as_id32());
			}
			PlayerIdentifier::Name(name) => {
				query
					.push(" owner.name LIKE ")
					.push_bind(format!(r#"%{name}%"#));
			}
		};

		clause = " AND ";
	}

	if let Some(approver) = params.approved_by {
		query.push(clause);
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

	query
		.push(" LIMIT ")
		.push_bind(params.limit.unwrap_or(500).min(500));

	let result: Vec<_> = query
		.build_query_as::<ServerRow>()
		.fetch_all(&global_state.conn)
		.await?
		.into_iter()
		.map(Into::into)
		.collect();

	debug!("Database result: {result:#?}");

	if result.is_empty() {
		return Err(Error::Database {
			kind: DatabaseError::NoRows,
		});
	}

	Ok(Json(result))
}
