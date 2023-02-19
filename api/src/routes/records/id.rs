use {
	crate::{
		models::{Response, ResponseBody},
		GlobalState,
	},
	axum::{
		extract::{Path, State},
		Json,
	},
	chrono::Utc,
	database::schemas::{FancyRecord, Record},
	log::debug,
};

pub(crate) async fn get(
	Path(record_id): Path<u32>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<FancyRecord> {
	let start = Utc::now().timestamp_nanos();
	debug!("[records::id::get]");
	debug!("> `record_id`: {record_id:#?}");

	let query = format!(
		r#"
		SELECT
		  record.id AS id,
		  course.map_id AS map_id,
		  map.name AS map_name,
		  map.courses AS map_courses,
		  map.validated AS map_validated,
		  map.filesize AS map_filesize,
		  map.created_by AS map_created_by_id,
		  mapper.name AS map_created_by_name,
		  mapper.is_banned AS map_created_by_is_banned,
		  map.approved_by AS map_approved_by_id,
		  approver.name AS map_approved_by_name,
		  approver.is_banned AS map_approved_by_is_banned,
		  map.created_on AS map_created_on,
		  map.updated_on AS map_updated_on,
		  course.id AS course_id,
		  course.stage AS course_stage,
		  course.kzt AS course_kzt,
		  course.kzt_difficulty AS course_kzt_difficulty,
		  course.skz AS course_skz,
		  course.skz_difficulty AS course_skz_difficulty,
		  course.vnl AS course_vnl,
		  course.vnl_difficulty AS course_vnl_difficulty,
		  mode.name AS mode_name,
		  player.name AS player_name,
		  player.id AS player_id,
		  server.name AS server_name,
		  record.time AS time,
		  record.teleports AS teleports,
		  record.created_on AS created_on
		FROM records AS record
		JOIN courses AS course ON course.id = record.course_id
		JOIN maps AS map ON map.id = course.map_id
		JOIN players AS mapper ON map.created_by = mapper.id
		JOIN players AS approver ON map.approved_by = approver.id
		JOIN modes AS mode ON mode.id = record.mode_id
		JOIN players AS player ON player.id = record.player_id
		JOIN servers AS server ON server.id = record.server_id
		WHERE record.id = {record_id}
		"#,
	);

	let record = sqlx::query_as::<_, Record>(&query)
		.fetch_one(&pool)
		.await?;

	Ok(Json(ResponseBody {
		result: FancyRecord::from(record),
		took: (Utc::now().timestamp_nanos() - start) as f64 / 1_000_000f64,
	}))
}
