mod id;
pub use id::id;

mod recent;
pub use recent::recent;

use {
	crate::{
		models::{
			records::{RecordModel, RecordResponse, RecordsQuery},
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
	query: Query<RecordsQuery>,
	state: State<GlobalState>,
) -> Result<Json<APIResponse<Vec<RecordResponse>>>, Error> {
	_index(query, state, false).await
}

async fn _index(
	Query(RecordsQuery { player_id, mode_id, has_teleports, stage }): Query<RecordsQuery>,
	State(GlobalState { pool }): State<GlobalState>,
	recent: bool,
) -> Result<Json<APIResponse<Vec<RecordResponse>>>, Error> {
	let start = Utc::now().timestamp_nanos();
	let mut base_query = QueryBuilder::<MySql>::new("SELECT * FROM records");
	let mut query = QueryBuilder::<MySql>::new("");

	if let Some(player_id) = player_id {
		query.push(" AND player_id = ");
		query.push(player_id);
	}

	if let Some(mode_id) = mode_id {
		query.push(" AND mode_id = ");
		query.push(mode_id);
	}

	if let Some(has_teleports) = has_teleports {
		query.push(if has_teleports { " AND teleports > 0" } else { " AND teleports = 0" });
	}

	if let Some(stage) = stage {
		query.push(" AND stage = ");
		query.push(stage);
	}

	if recent {
		query.push(" ORDER BY created_on ");
	}

	let query = if query.sql().is_empty() {
		base_query.into_sql()
	} else {
		base_query.push(query.into_sql().replacen("AND", "WHERE", 1));
		base_query.into_sql()
	};

	let records = sqlx::query_as::<_, RecordModel>(&query).fetch_all(&pool).await?;

	Ok(Json(APIResponse {
		result: records.into_iter().map(Into::into).collect(),
		took: Utc::now().timestamp_nanos() - start,
	}))
}
