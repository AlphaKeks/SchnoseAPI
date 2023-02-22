use {
	super::Record,
	crate::{routes::maps::Course, GlobalState, Response, ResponseBody},
	axum::{
		extract::{Path, State},
		Json,
	},
	database::schemas::{account_id_to_steam_id64, FancyPlayer},
	gokz_rs::prelude::*,
	log::debug,
	std::time::Instant,
};

pub(crate) async fn get(
	Path(record_id): Path<u32>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Record> {
	let start = Instant::now();
	debug!("[records::id::get]");
	debug!("> `record_id`: {record_id:#?}");

	let record_query = sqlx::query!(
		r#"
		SELECT
		  r.id AS id,
		  map.id AS map_id,
		  map.name AS map_name,
		  c.id AS course_id,
		  c.stage AS stage,
		  c.kzt AS `kzt: bool`,
		  c.kzt_difficulty AS kzt_difficulty,
		  c.skz AS `skz: bool`,
		  c.skz_difficulty AS skz_difficulty,
		  c.vnl AS `vnl: bool`,
		  c.vnl_difficulty AS vnl_difficulty,
		  mode.name AS mode,
		  p.id AS player_id,
		  p.name AS player_name,
		  p.is_banned AS `player_is_banned: bool`,
		  s.name AS server_name,
		  r.time AS time,
		  r.teleports AS teleports,
		  r.created_on AS created_on
		FROM records AS r
		JOIN courses AS c ON c.id = r.course_id
		JOIN maps AS map ON map.id = c.map_id
		JOIN modes AS mode ON mode.id = r.mode_id
		JOIN players AS p ON p.id = r.player_id
		JOIN servers AS s ON s.id = r.server_id
		WHERE r.id = ?
		"#,
		record_id
	)
	.fetch_one(&pool)
	.await?;

	let steam_id64 = account_id_to_steam_id64(record_query.player_id);
	let steam_id = SteamID::from(steam_id64);

	let result = Record {
		id: record_query.id,
		map_name: record_query.map_name,
		course: Course {
			id: record_query.course_id,
			stage: record_query.stage,
			kzt: record_query.kzt,
			kzt_difficulty: record_query.kzt_difficulty,
			skz: record_query.skz,
			skz_difficulty: record_query.skz_difficulty,
			vnl: record_query.vnl,
			vnl_difficulty: record_query.vnl_difficulty,
		},
		mode: record_query.mode,
		player: FancyPlayer {
			id: record_query.player_id,
			name: record_query.player_name,
			steam_id: steam_id.to_string(),
			steam_id64: steam_id64.to_string(),
			is_banned: record_query.player_is_banned,
		},
		server_name: record_query.server_name,
		time: record_query.time,
		teleports: record_query.teleports,
		created_on: record_query.created_on.to_string(),
	};

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
