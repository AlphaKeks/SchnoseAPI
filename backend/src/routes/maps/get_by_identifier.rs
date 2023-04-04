use {
	crate::GlobalState,
	axum::{
		extract::{Path, State},
		Json,
	},
	backend::{
		models::maps::{MapResponse, MapRow},
		Response, ResponseBody,
	},
	gokz_rs::MapIdentifier,
	tokio::time::Instant,
	tracing::debug,
};

pub async fn get_by_identifier(
	Path(map_identifier): Path<MapIdentifier>,
	State(global_state): State<GlobalState>,
) -> Response<MapResponse> {
	let took = Instant::now();
	debug!("[maps::get_by_identifier]");
	debug!("> `map_identifier`: {map_identifier:#?}");

	let map_id = database::select::get_map(map_identifier, &global_state.conn)
		.await?
		.id;

	let result: MapResponse = sqlx::query_as::<_, MapRow>(&format!(
		r#"
		SELECT
		  map.*,
		  mapper.id AS mapper_id,
		  mapper.name AS mapper_name,
		  approver.id AS approver_id,
		  approver.name AS approver_name,
		  JSON_ARRAYAGG(
		    JSON_OBJECT(
		      "id", courses.id,
		      "map_id", courses.map_id,
		      "stage", courses.stage,
		      "kzt", courses.kzt,
		      "kzt_difficulty", courses.kzt_difficulty,
		      "skz", courses.skz,
		      "skz_difficulty", courses.skz_difficulty,
		      "vnl", courses.vnl,
		      "vnl_difficulty", courses.vnl_difficulty
		    )
		  ) AS json_courses
		FROM maps AS map
		JOIN courses AS courses ON courses.map_id = map.id
		JOIN players AS mapper ON mapper.id = map.created_by
		JOIN players AS approver ON approver.id = map.approved_by
		WHERE map.id = {map_id}
		LIMIT 1
		"#
	))
	.fetch_one(&global_state.conn)
	.await?
	.try_into()?;

	debug!("Database result: {result:#?}");

	Ok(Json(ResponseBody {
		result,
		took: took.elapsed().as_nanos(),
	}))
}
