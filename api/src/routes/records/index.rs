use {
	crate::{
		models::{Response, ResponseBody},
		GlobalState,
	},
	axum::{
		extract::{Query, State},
		Json,
	},
	chrono::Utc,
	database::{
		crd::read::*,
		schemas::{FancyRecord, Record},
	},
	gokz_rs::prelude::*,
	log::debug,
	serde::Deserialize,
};

#[derive(Debug, Deserialize)]
pub(crate) struct Params {
	pub(crate) map: Option<String>,
	pub(crate) stage: Option<u8>,
	pub(crate) mode: Option<String>,
	pub(crate) player: Option<String>,
	pub(crate) server: Option<String>,
	pub(crate) has_teleports: Option<bool>,
	pub(crate) limit: Option<u32>,
}

pub(crate) async fn get(
	Query(params): Query<Params>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Vec<FancyRecord>> {
	let start = Utc::now().timestamp_nanos();
	debug!("[records::get]");
	debug!("> `params`: {params:#?}");

	let mut query = String::from(
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
		"#,
	);

	if let Some(map_ident) = params.map {
		let map_ident = map_ident.parse::<MapIdentifier>()?;
		let m = database::crd::read::get_map(&map_ident, &pool).await?;
		query.push_str(&format!("AND course.map_id = {} ", m.id));
	}

	if let Some(stage) = params.stage {
		query.push_str(&format!("AND course.stage = {stage} "));
	}

	if let Some(mode_input) = params.mode {
		let m = mode_input.parse::<Mode>()?;
		query.push_str(&format!("AND record.mode_id = {} ", m as u8));
	}

	if let Some(player_input) = params.player {
		let player_ident = player_input.parse::<PlayerIdentifier>()?;
		let p = database::crd::read::get_player_raw(player_ident, &pool).await?;
		query.push_str(&format!("AND record.player_id = {} ", p.id));
	}

	if let Some(server_input) = params.server {
		let s = database::crd::read::get_servers(
			QueryInput::Query(format!(
				r#"
				SELECT * FROM servers
				WHERE {}
				"#,
				if let Ok(server_id) = server_input.parse::<u16>() {
					format!("id = {server_id}")
				} else {
					format!(r#"name LIKE "%{server_input}%""#)
				}
			)),
			&pool,
		)
		.await?
		.remove(0);
		query.push_str(&format!("AND record.server_id = {} ", s.id));
	}

	if let Some(has_teleports) = params.has_teleports {
		query.push_str(&format!(
			"AND record.teleports {} 0 ",
			if has_teleports { ">" } else { "=" }
		));
	}

	query.push_str(&format!(
		"LIMIT {}",
		params
			.limit
			.map_or(100, |limit| limit.min(500))
	));
	query = query.replacen("AND", "WHERE", 1);

	let records = sqlx::query_as::<_, Record>(&query)
		.fetch_all(&pool)
		.await?
		.into_iter()
		.map(FancyRecord::from)
		.collect();

	Ok(Json(ResponseBody {
		result: records,
		took: (Utc::now().timestamp_nanos() - start) as f64 / 1_000_000f64,
	}))
}
