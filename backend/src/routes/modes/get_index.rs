use {
	crate::{DatabaseError, Error, GlobalState},
	axum::{extract::State, Json},
	backend::{Response, ResponseBody},
	database::schemas::ModeRow,
	log::debug,
	tokio::time::Instant,
};

pub async fn get_index(State(global_state): State<GlobalState>) -> Response<Vec<ModeRow>> {
	let took = Instant::now();
	debug!("[modes::get_index]");

	let result: Vec<ModeRow> = sqlx::query_as("SELECT * FROM modes")
		.fetch_all(&global_state.conn)
		.await?;

	debug!("Database result: {result:#?}");

	if result.is_empty() {
		return Err(Error::Database {
			kind: DatabaseError::NoRows,
		});
	}

	Ok(Json(ResponseBody {
		result,
		took: took.elapsed().as_nanos(),
	}))
}
