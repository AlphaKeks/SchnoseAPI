use {
	crate::{
		models::{
			servers::{ServerModel, ServerResponse},
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
	Path(id): Path<u16>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Result<Json<APIResponse<ServerResponse>>, Error> {
	let start = Utc::now().timestamp_nanos();
	let server =
		sqlx::query_as::<_, ServerModel>(&format!("SELECT * FROM servers WHERE id = {id}"))
			.fetch_one(&pool)
			.await?;

	Ok(Json(APIResponse {
		result: server.into(),
		took: Utc::now().timestamp_nanos() - start,
	}))
}
