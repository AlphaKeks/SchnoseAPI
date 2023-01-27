mod id;
pub use id::id;

mod name;
pub use name::name;

use {
	crate::{
		models::{
			players::{PlayerModel, PlayerQuery, PlayerResponse},
			APIResponse, Error,
		},
		GlobalState,
	},
	axum::{
		extract::{Query, State},
		Json,
	},
	chrono::Utc,
};

pub async fn index(
	Query(PlayerQuery { limit }): Query<PlayerQuery>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Result<Json<APIResponse<Vec<PlayerResponse>>>, Error> {
	let start = Utc::now().timestamp_nanos();
	let limit = limit.unwrap_or(1);
	let players = sqlx::query_as::<_, PlayerModel>(&format!("SELECT * FROM players LIMIT {limit}"))
		.fetch_all(&pool)
		.await?;

	Ok(Json(APIResponse {
		result: players.into_iter().map(Into::into).collect(),
		took: Utc::now().timestamp_nanos() - start,
	}))
}
