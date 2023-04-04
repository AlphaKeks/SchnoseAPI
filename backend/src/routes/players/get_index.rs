use {
	crate::{DatabaseError, Error, GlobalState},
	axum::{
		extract::{Query, State},
		Json,
	},
	backend::{models::players::PlayerParams, Response, ResponseBody},
	database::schemas::PlayerRow,
	sqlx::QueryBuilder,
	tokio::time::Instant,
	tracing::debug,
};

pub async fn get_index(
	Query(params): Query<PlayerParams>,
	State(global_state): State<GlobalState>,
) -> Response<Vec<PlayerRow>> {
	let took = Instant::now();
	debug!("[players::get_index]");
	debug!("> {params:#?}");

	let mut query = QueryBuilder::new("SELECT * FROM players");

	if let Some(is_banned) = params.is_banned {
		query
			.push(" WHERE is_banned = ")
			.push_bind(is_banned as u8);
	}

	query
		.push(" LIMIT ")
		.push_bind(params.limit.unwrap_or(500).min(500));

	if let Some(offset) = params.offset {
		query.push(" OFFSET ").push_bind(offset);
	}

	let result = query
		.build_query_as::<PlayerRow>()
		.fetch_all(&global_state.conn)
		.await?;

	debug!("Database result: {result:#?}");

	if result.is_empty() {
		return Err(Error::Database {
			kind: DatabaseError::NoRows,
		});
	}

	Ok(Json(ResponseBody {
		result: result
			.into_iter()
			.map(Into::into)
			.collect(),
		took: took.elapsed().as_nanos(),
	}))
}
