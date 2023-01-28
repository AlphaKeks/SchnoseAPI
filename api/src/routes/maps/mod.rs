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
	let mut base_query = QueryBuilder::<sqlx::MySql>::new(
		r#"
		SELECT
		  map.id,
		  map.name,
		  map.difficulty,
		  map.validated,
		  map.filesize,
		  map.created_by AS created_by_id,
		  mapper.name AS created_by_name,
		  map.approved_by AS approved_by_id,
		  approver.name AS approved_by_name,
		  map.created_on,
		  map.updated_on
		FROM maps AS map
		JOIN players AS mapper ON mapper.id = map.created_by
		JOIN players AS approver ON approver.id = map.approved_by
		"#,
	);
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

	let maps = sqlx::query_as::<_, MapModel>(&dbg!(query)).fetch_all(&pool).await?;

	Ok(Json(APIResponse {
		result: maps.into_iter().map(Into::into).collect(),
		took: Utc::now().timestamp_nanos() - start,
	}))
}
