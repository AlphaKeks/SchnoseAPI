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

pub async fn name(
	Path(name): Path<String>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Result<Json<APIResponse<MapResponse>>, Error> {
	let start = Utc::now().timestamp_nanos();
	let map = sqlx::query_as::<_, MapModel>(&format!(
		r#"SELECT * FROM maps WHERE name LIKE "%{name}%" LIMIT 1"#
	))
	.fetch_one(&pool)
	.await?;

	Ok(Json(APIResponse {
		result: map.into(),
		took: Utc::now().timestamp_nanos() - start,
	}))
}
