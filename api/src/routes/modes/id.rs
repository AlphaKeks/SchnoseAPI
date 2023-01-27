use {
	crate::{
		models::{
			modes::{ModeModel, ModeResponse},
			APIResponse, Error,
		},
		GlobalState,
	},
	axum::{
		extract::{Path, State},
		Json,
	},
	chrono::Utc,
};

pub async fn id(
	Path(id): Path<u8>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Result<Json<APIResponse<ModeResponse>>, Error> {
	let start = Utc::now().timestamp_nanos();
	let mode = sqlx::query_as::<_, ModeModel>(&format!("SELECT * FROM modes WHERE id = {id}"))
		.fetch_one(&pool)
		.await?;

	Ok(Json(APIResponse {
		result: mode.into(),
		took: Utc::now().timestamp_nanos() - start,
	}))
}
