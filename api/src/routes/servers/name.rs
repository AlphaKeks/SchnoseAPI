use {
	crate::{
		models::{
			servers::{ServerModel, ServerResponse},
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
) -> Result<Json<APIResponse<ServerResponse>>, Error> {
	let start = Utc::now().timestamp_nanos();
	let server = sqlx::query_as::<_, ServerModel>(&format!(
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
		WHERE server.name LIKE "%{name}%"
		"#,
	))
	.fetch_one(&pool)
	.await?;

	Ok(Json(APIResponse {
		result: server.into(),
		took: Utc::now().timestamp_nanos() - start,
	}))
}
