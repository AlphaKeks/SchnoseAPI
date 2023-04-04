use {
	crate::{DatabaseError, Error, GlobalState},
	axum::{extract::State, Json},
	backend::Response,
	database::schemas::ModeRow,
	tracing::debug,
};

pub async fn get_index(State(global_state): State<GlobalState>) -> Response<Vec<ModeRow>> {
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

	Ok(Json(result))
}
