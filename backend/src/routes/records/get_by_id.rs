use {
	crate::GlobalState,
	axum::{
		extract::{Path, State},
		Json,
	},
	backend::{
		models::records::{RecordResponse, RecordRow},
		Response,
	},
	sqlx::QueryBuilder,
	tracing::debug,
};

pub async fn get_by_id(
	Path(record_id): Path<u32>,
	State(global_state): State<GlobalState>,
) -> Response<RecordResponse> {
	debug!("[records::get_by_id]");
	debug!("> `record_id`: {record_id:#?}");

	let mut query = QueryBuilder::new(
		r#"
		SELECT
		  record.id AS id,
		  player.name AS player_name,
		  player.id AS player_id,
		  course.map_id AS map_id,
		  map.name AS map_name,
		  course.stage AS stage,
		  record.mode_id AS mode_id,
		  record.time AS time,
		  record.teleports AS teleports,
		  record.created_on AS created_on
		FROM records AS record
		JOIN players AS player ON player.id = record.player_id
		JOIN courses AS course ON course.id = record.course_id
		JOIN maps AS map ON map.id = course.map_id
		WHERE record.id = 
		"#,
	);

	query.push_bind(record_id);

	let result = query
		.build_query_as::<RecordRow>()
		.fetch_one(&global_state.conn)
		.await?
		.try_into()?;

	debug!("Database result: {result:#?}");

	Ok(Json(result))
}
