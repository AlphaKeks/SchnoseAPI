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

	let mut query = QueryBuilder::<MySql>::new(
		r#"
		SELECT
		  server.id,
		  server.name,
		  server.owner_id,
		  owner.name AS owner_name,
		  server.approved_by AS approved_by_id,
		  approver.name AS approved_by_name,
		  server.approved_on,
		  server.updated_on
		FROM servers AS server
		JOIN players AS owner ON server.owner_id = owner.id
		JOIN players AS approver ON server.approved_by = approver.id
		"#,
	);

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
