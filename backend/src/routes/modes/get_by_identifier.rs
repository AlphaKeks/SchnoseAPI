use {
	crate::GlobalState,
	axum::{
		extract::{Path, State},
		Json,
	},
	backend::Response,
	database::schemas::ModeRow,
	gokz_rs::Mode,
	tracing::debug,
};

pub async fn get_by_identifier(
	Path(mode): Path<Mode>,
	State(global_state): State<GlobalState>,
) -> Response<ModeRow> {
	debug!("[modes::get_by_identifier]");
	debug!("> `mode`: {mode:#?}");

	let result: ModeRow = sqlx::query_as(&format!("SELECT * FROM modes WHERE id = {}", mode as u8))
		.fetch_one(&global_state.conn)
		.await?;

	debug!("Database result: {result:#?}");

	Ok(Json(result))
}
