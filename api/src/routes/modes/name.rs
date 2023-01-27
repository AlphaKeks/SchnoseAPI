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

pub async fn name(
	Path(name): Path<String>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Result<Json<APIResponse<ModeResponse>>, Error> {
	let start = Utc::now().timestamp_nanos();
	let mode = sqlx::query_as::<_, ModeModel>(&format!(
		r#"SELECT * FROM modes WHERE name LIKE "%{name}%" OR name_short LIKE "%{name}%" OR name_long LIKE "%{name}" LIMIT 1"#
	))
	.fetch_one(&pool)
	.await?;

	Ok(Json(APIResponse {
		result: mode.into(),
		took: Utc::now().timestamp_nanos() - start,
	}))
}
