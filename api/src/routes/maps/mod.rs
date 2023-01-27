mod id;
pub use id::id;

mod name;
pub use name::name;
use sqlx::QueryBuilder;

use {
	crate::{
		models::{
			maps::{MapModel, MapQuery, MapResponse},
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
	Query(MapQuery { name, validated, created_by, created_on, limit }): Query<MapQuery>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Result<Json<APIResponse<Vec<MapResponse>>>, Error> {
	let start = Utc::now().timestamp_nanos();
	let mut base_query = QueryBuilder::<sqlx::MySql>::new("SELECT * FROM maps");
	let mut query = QueryBuilder::<sqlx::MySql>::new("");

	if let Some(name) = name {
		query.push(r#" AND name LIKE "%"#);
		query.push(name);
		query.push(r#"%""#);
	}

	if let Some(validated) = validated {
		query.push(" AND validated = ");
		query.push(validated);
	}

	if let Some(created_by) = created_by {
		query.push(" AND created_by = ");
		query.push(created_by);
	}

	if let Some(created_on) = created_on {
		query.push(" AND created_on = ");
		query.push(created_on);
	}

	if let Some(limit) = limit {
		query.push(" LIMIT ");
		query.push(limit);
	}

	let query = if query.sql().is_empty() {
		base_query.into_sql()
	} else {
		base_query.push(query.into_sql().replacen("AND", "WHERE", 1));
		base_query.into_sql()
	};

	let players = sqlx::query_as::<_, MapModel>(&dbg!(query)).fetch_all(&pool).await?;

	Ok(Json(APIResponse {
		result: players.into_iter().map(Into::into).collect(),
		took: Utc::now().timestamp_nanos() - start,
	}))
}
