use database::crd::read::get_map;

use {
	super::{PlayerRowJSON, Record, RecordQuery},
	crate::{
		models::{Response, ResponseBody},
		routes::maps::Course,
		GlobalState,
	},
	axum::{
		extract::{Path, Query, State},
		Json,
	},
	database::{
		crd::read::get_player,
		schemas::{account_id_to_steam_id64, FancyPlayer},
	},
	gokz_rs::prelude::*,
	log::debug,
	serde::Deserialize,
	std::time::Instant,
};

#[derive(Debug, Deserialize)]
pub struct Params {
	mode: Option<String>,
	stage: Option<u8>,
	map: Option<String>,
	has_teleports: Option<bool>,
	limit: Option<u32>,
}

pub(crate) async fn get(
	Path(player_ident): Path<String>,
	Query(params): Query<Params>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Vec<Record>> {
	let start = Instant::now();
	debug!("[records::player::get]");
	debug!("> `player_ident`: {player_ident:#?}");
	debug!("> `params`: {params:#?}");

	let player_ident = player_ident.parse::<PlayerIdentifier>()?;
	debug!("> `player_ident`: {player_ident:#?}");

	let player_id = get_player(player_ident, &pool)
		.await
		.map(|player_row| player_row.id)?;

	let mut filter = String::new();

	if let Some(mode) = params.mode {
		let mode = mode.parse::<Mode>()?;
		filter.push_str(&format!("AND mode_id = {} ", mode as u8));
	}

	let stage_filter = params
		.stage
		.map_or(String::new(), |stage| format!("AND c.stage = {stage} "));

	let map_filter = if let Some(map) = params.map {
		let map_ident = map.parse::<MapIdentifier>()?;
		let map = get_map(map_ident, &pool).await?;
		map.id.to_string()
	} else {
		String::new()
	};

	if let Some(has_teleports) = params.has_teleports {
		filter.push_str(&format!(
			"AND teleports {} 0 ",
			if has_teleports { ">" } else { "=" }
		));
	}

	filter = filter.replacen("AND", "WHERE", 1);

	let limit = params
		.limit
		.map_or(100, |limit| limit.min(250));

	let mut result = Vec::new();
	for record_query in sqlx::query_as::<_, RecordQuery>(&format!(
		r#"
		SELECT
		  r.id AS id,
		  ma.name AS map_name,
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
		  mo.name AS mode,
		  JSON_OBJECT(
		    "id", p.id,
		    "name", p.name,
		    "is_banned", p.is_banned
		  ) AS player,
		  s.name AS server_name,
		  r.time AS time,
		  r.teleports AS teleports,
		  r.created_on AS created_on
		FROM (
		  SELECT * FROM records
		  {filter}
		  ORDER BY created_on DESC
		) AS r
		JOIN courses AS c ON c.id = r.course_id {stage_filter}
		JOIN maps AS ma ON ma.id = c.map_id AND ma.id = {map_filter}
		JOIN modes AS mo ON mo.id = r.mode_id
		JOIN players AS p ON p.id = r.player_id AND p.is_banned = 0 AND p.id = {player_id}
		JOIN servers AS s ON s.id = r.server_id
		ORDER BY r.time, r.created_on DESC
		LIMIT {limit}
		"#,
	))
	.fetch_all(&pool)
	.await?
	{
		let Ok(course) = serde_json::from_str::<Course>(&record_query.course) else {
			continue;
		};

		let Ok(player) = serde_json::from_str::<PlayerRowJSON>(&record_query.player) else {
			continue;
		};

		let steam_id64 = account_id_to_steam_id64(player.id);
		let steam_id = SteamID::from(steam_id64);

		result.push(Record {
			id: record_query.id,
			map_name: record_query.map_name,
			course,
			mode: record_query.mode,
			player: FancyPlayer {
				id: player.id,
				name: player.name,
				steam_id: steam_id.to_string(),
				steam_id64: steam_id64.to_string(),
				is_banned: player.is_banned,
			},
			server_name: record_query.server_name,
			time: record_query.time,
			teleports: record_query.teleports,
			created_on: record_query.created_on.to_string(),
		});
	}

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
