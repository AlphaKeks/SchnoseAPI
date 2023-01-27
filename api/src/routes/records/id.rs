use {
	crate::{
		models::{
			records::{RecordModel, RecordResponse},
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
	Path(id): Path<u32>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Result<Json<APIResponse<RecordResponse>>, Error> {
	let start = Utc::now().timestamp_nanos();
	let record =
		sqlx::query_as::<_, RecordModel>(&format!("SELECT * FROM records WHERE id = {id}"))
			.fetch_one(&pool)
			.await?;

	Ok(Json(APIResponse {
		result: record.into(),
		took: Utc::now().timestamp_nanos() - start,
	}))
}
