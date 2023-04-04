use {
	crate::GlobalState,
	axum::{
		extract::{Path, State},
		Json,
	},
	backend::{
		models::servers::{ServerResponse, ServerRow},
		Response,
	},
	gokz_rs::ServerIdentifier,
	tracing::debug,
};

pub async fn get_by_identifier(
	Path(server_identifier): Path<ServerIdentifier>,
	State(global_state): State<GlobalState>,
) -> Response<ServerResponse> {
	debug!("[servers::get_by_identifier]");
	debug!("> `server_identifier`: {server_identifier:#?}");

	let server_id = database::select::get_server(server_identifier, &global_state.conn)
		.await?
		.id;

	let result: ServerResponse = sqlx::query_as::<_, ServerRow>(&format!(
		r#"
		SELECT
		  server.id AS id,
		  server.name AS name,
		  owner.id AS owner_id,
		  owner.name AS owner_name,
		  approver.id AS approver_id,
		  approver.name AS approver_name
		FROM servers AS server
		JOIN players AS owner ON owner.id = server.owned_by
		JOIN players AS approver ON approver.id = server.approved_by
		WHERE server.id = {server_id}
		LIMIT 1
		"#
	))
	.fetch_one(&global_state.conn)
	.await?
	.into();

	debug!("Database result: {result:#?}");

	Ok(Json(result))
}
