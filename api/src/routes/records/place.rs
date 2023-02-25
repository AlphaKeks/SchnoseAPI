use {
	crate::{Error, GlobalState, Response, ResponseBody},
	axum::{
		extract::{Path, State},
		Json,
	},
	database::{
		crd::read::{get_course, get_record},
		schemas::{CourseRow, RecordRow},
	},
	log::debug,
	std::time::Instant,
};

pub(crate) async fn get(
	Path(record_id): Path<u32>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<u32> {
	let start = Instant::now();
	debug!("[records::id::get]");
	debug!("> `record_id`: {record_id:#?}");

	let RecordRow {
		id,
		course_id,
		mode_id,
		teleports,
		..
	} = get_record(record_id, &pool).await?;

	let CourseRow { map_id, stage, .. } = get_course(course_id, &pool).await?;

	let (if_no_teleports, if_teleports) = if teleports > 0 { (0, 1) } else { (1, 0) };

	let result = sqlx::query!(
		r#"
		SELECT
		  r.id AS id,
		  map.id AS map_id,
		  map.name AS map_name,
		  c.id AS course_id,
		  c.stage AS stage,
		  c.kzt AS kzt,
		  c.kzt_difficulty AS kzt_difficulty,
		  c.skz AS skz,
		  c.skz_difficulty AS skz_difficulty,
		  c.vnl AS vnl,
		  c.vnl_difficulty AS vnl_difficulty,
		  mode.name AS mode,
		  p.id AS player_id,
		  p.name AS player_name,
		  p.is_banned AS player_is_banned,
		  s.name AS server_name,
		  r.time AS time,
		  r.teleports AS teleports,
		  r.created_on AS created_on
		FROM (
		  SELECT
		    r_inner.mode_id,
		    r_inner.course_id,
		    r_inner.player_id,
		    CASE WHEN r_inner.teleports = 0 THEN ? ELSE ? END AS has_teleports,
		    MIN(r_inner.time) AS time
		  FROM records AS r_inner
		  JOIN courses AS c ON c.id = r_inner.course_id
		    AND c.stage = ?
		    AND c.map_id = ?
		  JOIN modes AS mode ON mode.id = r_inner.mode_id AND mode.id = ?
		  WHERE CASE WHEN r_inner.teleports = 0 THEN ? ELSE ? END
		  GROUP BY
		    r_inner.mode_id,
		    r_inner.course_id,
		    r_inner.player_id,
		    has_teleports
		) AS pb
		JOIN records AS r ON r.mode_id = pb.mode_id
		AND r.course_id = pb.course_id
		AND r.player_id = pb.player_id
		AND r.time = pb.time
		JOIN courses AS c ON c.id = r.course_id
		JOIN maps AS map ON map.id = c.map_id AND c.map_id = ? AND c.stage = ?
		JOIN modes AS mode ON mode.id = r.mode_id AND r.mode_id = ?
		JOIN players AS p ON p.id = r.player_id
		JOIN servers AS s ON s.id = r.server_id
		GROUP BY
		  r.mode_id,
		  r.course_id,
		  r.player_id,
		  pb.has_teleports
		ORDER BY
		  c.stage ASC,
		  r.time,
		  r.created_on DESC
		"#,
		if_no_teleports,
		if_teleports,
		stage,
		map_id,
		mode_id,
		if teleports == 0 { if_no_teleports } else { if_teleports },
		if teleports == 0 { if_teleports } else { if_no_teleports },
		map_id,
		stage,
		mode_id
	)
	.fetch_all(&pool)
	.await?;

	let result = result
		.into_iter()
		.enumerate()
		.find_map(|(i, record)| {
			if record.id == id {
				return Some(i as u32 + 1u32);
			}
			None
		})
		.ok_or(Error::Database {
			// FIXME: This should never happen but sometimes it does.
			message: String::from("No place found."),
		})?;

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
