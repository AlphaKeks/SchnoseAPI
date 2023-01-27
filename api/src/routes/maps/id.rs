use {
	crate::{
		models::{
			maps::{MapModel, MapResponse},
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
	Path(id): Path<u64>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Result<Json<APIResponse<MapResponse>>, Error> {
	let start = Utc::now().timestamp_nanos();
	let player = sqlx::query_as::<_, MapModel>(&format!("SELECT * FROM maps WHERE id = {id}"))
		.fetch_one(&pool)
		.await?;

	Ok(Json(APIResponse {
		result: player.into(),
		took: Utc::now().timestamp_nanos() - start,
	}))
}
