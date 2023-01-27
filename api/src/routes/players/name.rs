use {
	crate::{
		models::{
			players::{PlayerModel, PlayerResponse},
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
) -> Result<Json<APIResponse<PlayerResponse>>, Error> {
	let start = Utc::now().timestamp_nanos();
	let player = sqlx::query_as::<_, PlayerModel>(&format!(
		r#"SELECT * FROM players WHERE name LIKE "%{name}%" LIMIT 1"#
	))
	.fetch_one(&pool)
	.await?;

	Ok(Json(APIResponse {
		result: player.into(),
		took: Utc::now().timestamp_nanos() - start,
	}))
}
