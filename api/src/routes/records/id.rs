use {
	super::{PlayerRowJSON, Record, RecordQuery},
	crate::{
		models::{Response, ResponseBody},
		routes::maps::Course,
		GlobalState,
	},
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

	let record_query = sqlx::query_as::<_, RecordQuery>(&format!(
		r#"
		SELECT
		  r.id AS id,
		  JSON_OBJECT(
		    "id", c.id,
		    "stage", c.stage,
		    "kzt", c.kzt,
		    "kzt_difficulty", c.kzt_difficulty,
		    "skz", c.skz,
		    "skz_difficulty", c.skz_difficulty,
		    "vnl", c.vnl,
		    "vnl_difficulty", c.vnl_difficulty
		  ) AS course,
		  m.name AS mode,
		  JSON_OBJECT(
		    "id", p.id,
		    "name", p.name,
		    "is_banned", p.is_banned
		  ) AS player,
		  s.name AS server_name,
		  r.time AS time,
		  r.teleports AS teleports,
		  r.created_on AS created_on
		FROM records AS r
		JOIN courses AS c ON c.id = r.course_id
		JOIN modes AS m ON m.id = r.mode_id
		JOIN players AS p ON p.id = r.player_id
		JOIN servers AS s ON s.id = r.server_id
		WHERE r.id = {record_id}
		"#
	))
	.fetch_one(&pool)
	.await?;

	let course = serde_json::from_str::<Course>(&record_query.course)
		// .map_err(|_| Error::JSON)?;
		.unwrap();
	let player = {
		let player = serde_json::from_str::<PlayerRowJSON>(&record_query.player)
			// .map_err(|_| Error::JSON)?;
			.unwrap();
		let steam_id64 = account_id_to_steam_id64(player.id);
		let steam_id = SteamID::from(steam_id64);
		FancyPlayer {
			id: player.id,
			name: player.name,
			steam_id: steam_id.to_string(),
			steam_id64: steam_id64.to_string(),
			is_banned: player.is_banned,
		}
	};

	let result = Record {
		id: record_query.id,
		course,
		mode: record_query.mode,
		player,
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
