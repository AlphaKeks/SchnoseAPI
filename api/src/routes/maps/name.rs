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
		WHERE map.name LIKE "%{name}%"
		"#,
	))
	.fetch_one(&pool)
	.await?;

	Ok(Json(APIResponse {
		result: map.into(),
		took: Utc::now().timestamp_nanos() - start,
	}))
}
