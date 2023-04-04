use {
	crate::GlobalState,
	axum::{
		extract::{Path, State},
		Json,
	},
	backend::{Response, ResponseBody},
	database::schemas::ModeRow,
	gokz_rs::Mode,
	tokio::time::Instant,
	tracing::debug,
};

pub async fn get_by_identifier(
	Path(mode): Path<Mode>,
	State(global_state): State<GlobalState>,
) -> Response<ModeRow> {
	let took = Instant::now();
	debug!("[modes::get_by_identifier]");
	debug!("> `mode`: {mode:#?}");

	let result: ModeRow = sqlx::query_as(&format!("SELECT * FROM modes WHERE id = {}", mode as u8))
		.fetch_one(&global_state.conn)
		.await?;

	debug!("Database result: {result:#?}");

	Ok(Json(ResponseBody {
		result,
		took: took.elapsed().as_nanos(),
	}))
}
