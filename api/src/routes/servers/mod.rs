mod id;
pub use id::id;

mod name;
pub use name::name;

use {
	crate::{
		models::{
			servers::{ServerModel, ServerQuery, ServerResponse},
			APIResponse, Error,
		},
		GlobalState,
	},
	axum::{
		extract::{Query, State},
		Json,
	},
	chrono::Utc,
	sqlx::{MySql, QueryBuilder},
};

pub async fn index(
	Query(ServerQuery { name, owner_id, limit }): Query<ServerQuery>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Result<Json<APIResponse<Vec<ServerResponse>>>, Error> {
	let start = Utc::now().timestamp_nanos();

	let mut query = QueryBuilder::<MySql>::new("SELECT * FROM servers ");

	if let Some(name) = name {
		query.push(r#"AND name LIKE "%"#);
		query.push(name);
		query.push(r#"%""#);
	}

	if let Some(owner_id) = owner_id {
		query.push("AND owner_id = ");
		query.push(owner_id);
	}

	if let Some(limit) = limit {
		query.push(" LIMIT ");
		query.push(limit);
	}

	let query = dbg!(query.into_sql().replacen("AND", "WHERE", 1));

	let servers = sqlx::query_as::<_, ServerModel>(&query).fetch_all(&pool).await?;

	Ok(Json(APIResponse {
		result: servers.into_iter().map(Into::into).collect(),
		took: Utc::now().timestamp_nanos() - start,
	}))
}
