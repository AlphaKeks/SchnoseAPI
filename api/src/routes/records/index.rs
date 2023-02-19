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
	color_eyre::eyre::eyre,
	color_eyre::Result as Eyre,
	database::{
		crd::read::*,
		schemas::{
			steam_id64_to_account_id, steam_id_to_account_id, CompactPlayer, FancyRecord,
			FullRecord, RecordWithoutMap, RecordWithoutMapOrPlayer,
			RecordWithoutMapOrPlayerOrServer, RecordWithoutMapOrServer, RecordWithoutPlayer,
			RecordWithoutServer, RecordWithoutServerOrPlayer, MAGIC_STEAM_ID_OFFSET,
		},
	},
	gokz_rs::prelude::*,
	log::debug,
	serde::Deserialize,
	sqlx::{MySql, Pool},
};

const BASE_QUERY: &str = r#"
		SELECT
		  record.id AS id,
		  course.map_id AS map_id,
		  course.id AS course_id,
		  course.stage AS course_stage,
		  course.kzt AS course_kzt,
		  course.kzt_difficulty AS course_kzt_difficulty,
		  course.skz AS course_skz,
		  course.skz_difficulty AS course_skz_difficulty,
		  course.vnl AS course_vnl,
		  course.vnl_difficulty AS course_vnl_difficulty,
		  mode.name AS mode_name,
		  record.time AS time,
		  record.teleports AS teleports,
		  record.created_on AS created_on,
"#;

const BASE_JOINS: &str = r#"
		FROM records AS record
		JOIN courses AS course ON course.id = record.course_id
		JOIN modes AS mode ON mode.id = record.mode_id
"#;

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

	let mut filter = String::new();

	if let Some(stage) = params.stage {
		filter.push_str(&format!("AND course.stage = {stage} "));
	}

	if let Some(mode) = params.mode {
		let mode = mode.parse::<Mode>()? as u8;
		filter.push_str(&format!("AND record.mode_id = {mode} "));
	}

	if let Some(has_teleports) = params.has_teleports {
		filter.push_str(&format!(
			"AND record.teleports {} 0 ",
			if has_teleports { ">" } else { "=" }
		));
	}

	let limit = params
		.limit
		.map_or(100, |limit| limit.min(500));

	let records = match (params.map, params.player, params.server) {
		(None, None, None) => no_params(filter, limit, &pool).await?,
		(None, None, Some(server)) => only_server(server, filter, limit, &pool).await?,
		(None, Some(player), None) => only_player(player, filter, limit, &pool).await?,
		(None, Some(player), Some(server)) => {
			player_server(player, server, filter, limit, &pool).await?
		}
		(Some(map), None, None) => only_map(map, filter, limit, &pool).await?,
		(Some(map), None, Some(server)) => map_server(map, server, filter, limit, &pool).await?,
		(Some(map), Some(player), None) => map_player(map, player, filter, limit, &pool).await?,
		(Some(map), Some(player), Some(server)) => {
			all_params(map, player, server, filter, limit, &pool).await?
		}
	};

	Ok(Json(ResponseBody {
		result: records,
		took: (Utc::now().timestamp_nanos() - start) as f64 / 1_000_000f64,
	}))
}

const NO_PARAMS: &str = r#"
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
		  player.name AS player_name,
		  player.id AS player_id,
		  server.name AS server_name
"#;
const NO_PARAMS_JOINS: &str = r#"
		JOIN maps AS map ON map.id = course.map_id
		JOIN players AS mapper ON map.created_by = mapper.id
		JOIN players AS approver ON map.approved_by = approver.id
		JOIN players AS player ON player.id = record.player_id
		JOIN servers AS server ON server.id = record.server_id
"#;
async fn no_params(mut filter: String, limit: u32, pool: &Pool<MySql>) -> Eyre<Vec<FancyRecord>> {
	filter = filter.replacen("AND", "WHERE", 1);

	let query = format!(
		r#"
		{BASE_QUERY}
		{NO_PARAMS}
		{BASE_JOINS}
		{NO_PARAMS_JOINS}
		{filter}
		ORDER BY record.created_on
		LIMIT {limit}
		"#
	);

	let mut records = Vec::new();
	for record in sqlx::query_as::<_, FullRecord>(&query)
		.fetch_all(pool)
		.await?
	{
		let course = get_course(record.course_id, pool).await?;
		let map = get_map(&MapIdentifier::ID(course.map_id as i32), pool).await?;
		let steam_id64 = record.player_id as u64 + MAGIC_STEAM_ID_OFFSET;
		let player = get_player_raw(PlayerIdentifier::SteamID64(steam_id64), pool).await?;

		records.push(FancyRecord {
			id: record.id,
			map,
			course,
			mode: record.mode_name,
			player: CompactPlayer {
				name: player.name,
				steam_id64: steam_id64.to_string(),
			},
			server: record.server_name,
			time: record.time,
			teleports: record.teleports,
			created_on: record.created_on.to_string(),
		});
	}

	Ok(records)
}

const ONLY_SERVER: &str = r#"
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
		  player.name AS player_name,
		  player.id AS player_id,
		  record.server_id AS server_id
"#;
const ONLY_SERVER_JOINS: &str = r#"
		JOIN maps AS map ON map.id = course.map_id
		JOIN players AS mapper ON map.created_by = mapper.id
		JOIN players AS approver ON map.approved_by = approver.id
		JOIN players AS player ON player.id = record.player_id
"#;
async fn only_server(
	server: String,
	mut filter: String,
	limit: u32,
	pool: &Pool<MySql>,
) -> Eyre<Vec<FancyRecord>> {
	filter.push_str(&format!(
		r#"
		AND {}
		"#,
		match server.parse::<u16>() {
			Ok(id) => format!("record.server_id = {id}"),
			_ => format!(r#"record.server_name LIKE "%{server}%""#),
		}
	));

	filter = filter.replacen("AND", "WHERE", 1);

	let query = format!(
		r#"
		{BASE_QUERY}
		{ONLY_SERVER}
		{BASE_JOINS}
		{ONLY_SERVER_JOINS}
		{filter}
		ORDER BY record.created_on
		LIMIT {limit}
		"#
	);

	let server = get_servers(
		QueryInput::Filter(format!(
			r#"
			WHERE {} {}
			"#,
			if server.parse::<u16>().is_ok() { "id =" } else { "name LIKE" },
			if let Ok(id) = server.parse::<u16>() {
				format!("{id}")
			} else {
				format!(r#""%{server}%""#)
			}
		)),
		pool,
	)
	.await?
	.into_iter()
	.map(|server| server.name)
	.collect::<Vec<_>>()
	.remove(0);

	let mut records = Vec::new();
	for record in sqlx::query_as::<_, RecordWithoutServer>(&query)
		.fetch_all(pool)
		.await?
	{
		let course = get_course(record.course_id, pool).await?;
		let map = get_map(&MapIdentifier::ID(course.map_id as i32), pool).await?;
		let steam_id64 = record.player_id as u64 + MAGIC_STEAM_ID_OFFSET;
		let player = get_player_raw(PlayerIdentifier::SteamID64(steam_id64), pool).await?;

		records.push(FancyRecord {
			id: record.id,
			map,
			course,
			mode: record.mode_name,
			player: CompactPlayer {
				name: player.name,
				steam_id64: steam_id64.to_string(),
			},
			server: server.clone(),
			time: record.time,
			teleports: record.teleports,
			created_on: record.created_on.to_string(),
		});
	}

	Ok(records)
}

const ONLY_PLAYER: &str = r#"
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
		  server.name AS server_name,
		  record.player_id AS player_id
"#;
const ONLY_PLAYER_JOINS: &str = r#"
		JOIN maps AS map ON map.id = course.map_id
		JOIN players AS mapper ON map.created_by = mapper.id
		JOIN players AS approver ON map.approved_by = approver.id
		JOIN servers AS server ON server.id = record.server_id
"#;
async fn only_player(
	player: String,
	mut filter: String,
	limit: u32,
	pool: &Pool<MySql>,
) -> Eyre<Vec<FancyRecord>> {
	let (maybe_player, f) = match player.parse::<PlayerIdentifier>()? {
		ident @ PlayerIdentifier::Name(_) => {
			let player = get_player_raw(ident, pool).await?;
			let f = format!(r#"record.player_id = {}"#, player.id);
			(Some(player), f)
		}
		PlayerIdentifier::SteamID(steam_id) => {
			let account_id =
				steam_id_to_account_id(&steam_id.to_string()).ok_or(eyre!("Invalid SteamID"))?;
			(None, format!(r#"record.player_id = {account_id}"#))
		}
		PlayerIdentifier::SteamID64(steam_id64) => {
			let account_id = steam_id64_to_account_id(steam_id64)?;
			(None, format!(r#"record.player_id = {account_id}"#))
		}
	};

	filter.push_str(&format!(
		r#"
		AND {f}
		"#,
	));

	filter = filter.replacen("AND", "WHERE", 1);

	let query = format!(
		r#"
		{BASE_QUERY}
		{ONLY_PLAYER}
		{BASE_JOINS}
		{ONLY_PLAYER_JOINS}
		{filter}
		ORDER BY record.created_on
		LIMIT {limit}
		"#
	);

	let player = if let Some(player) = maybe_player {
		player
	} else {
		get_player_raw(player.parse()?, pool).await?
	};

	let mut records = Vec::new();
	for record in sqlx::query_as::<_, RecordWithoutPlayer>(&query)
		.fetch_all(pool)
		.await?
	{
		let course = get_course(record.course_id, pool).await?;
		let map = get_map(&MapIdentifier::ID(course.map_id as i32), pool).await?;

		records.push(FancyRecord {
			id: record.id,
			map,
			course,
			mode: record.mode_name,
			player: CompactPlayer {
				name: player.name.clone(),
				steam_id64: (player.id as u64 + MAGIC_STEAM_ID_OFFSET).to_string(),
			},
			server: record.server_name,
			time: record.time,
			teleports: record.teleports,
			created_on: record.created_on.to_string(),
		});
	}

	Ok(records)
}

const PLAYER_SERVER: &str = r#"
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
		  record.player_id AS player_id,
		  record.server_id AS server_id
"#;
const PLAYER_SERVER_JOINS: &str = r#"
		JOIN maps AS map ON map.id = course.map_id
		JOIN players AS mapper ON map.created_by = mapper.id
		JOIN players AS approver ON map.approved_by = approver.id
"#;
async fn player_server(
	player: String,
	server: String,
	mut filter: String,
	limit: u32,
	pool: &Pool<MySql>,
) -> Eyre<Vec<FancyRecord>> {
	let (maybe_player, f) = match player.parse::<PlayerIdentifier>()? {
		ident @ PlayerIdentifier::Name(_) => {
			let player = get_player_raw(ident, pool).await?;
			let f = format!(r#"record.player_id = {}"#, player.id);
			(Some(player), f)
		}
		PlayerIdentifier::SteamID(steam_id) => {
			let account_id =
				steam_id_to_account_id(&steam_id.to_string()).ok_or(eyre!("Invalid SteamID"))?;
			(None, format!(r#"record.player_id = {account_id}"#))
		}
		PlayerIdentifier::SteamID64(steam_id64) => {
			let account_id = steam_id64_to_account_id(steam_id64)?;
			(None, format!(r#"record.player_id = {account_id}"#))
		}
	};

	filter.push_str(&format!(
		r#"
		AND {f}
		"#,
	));

	filter = filter.replacen("AND", "WHERE", 1);

	let (maybe_server, f) = match server.parse::<u16>() {
		Ok(server_id) => (None, format!("record.server_id = {server_id}")),
		_ => {
			let server = get_servers(
				QueryInput::Filter(format!(
					r#"
					WHERE {} {}
					"#,
					if server.parse::<u16>().is_ok() { "id =" } else { "name LIKE" },
					if let Ok(id) = server.parse::<u16>() {
						format!("{id}")
					} else {
						format!(r#""%{server}%""#)
					}
				)),
				pool,
			)
			.await?
			.remove(0);

			let f = format!("record.server_id = {}", server.id);

			(Some(server), f)
		}
	};

	filter.push_str(&format!(
		r#"
		AND {f}
		"#,
	));

	filter = filter.replacen("AND", "WHERE", 1);

	let query = format!(
		r#"
		{BASE_QUERY}
		{PLAYER_SERVER}
		{BASE_JOINS}
		{PLAYER_SERVER_JOINS}
		{filter}
		ORDER BY record.created_on
		LIMIT {limit}
		"#
	);

	let player = if let Some(player) = maybe_player {
		player
	} else {
		get_player_raw(player.parse()?, pool).await?
	};

	let server = if let Some(server) = maybe_server {
		server
	} else {
		get_servers(
			QueryInput::Filter(format!(
				r#"
				WHERE {} {}
				"#,
				if server.parse::<u16>().is_ok() { "id =" } else { "name LIKE" },
				if let Ok(id) = server.parse::<u16>() {
					format!("{id}")
				} else {
					format!(r#""%{server}%""#)
				}
			)),
			pool,
		)
		.await?
		.remove(0)
	};

	let mut records = Vec::new();
	for record in sqlx::query_as::<_, RecordWithoutServerOrPlayer>(&query)
		.fetch_all(pool)
		.await?
	{
		let course = get_course(record.course_id, pool).await?;
		let map = get_map(&MapIdentifier::ID(course.map_id as i32), pool).await?;

		records.push(FancyRecord {
			id: record.id,
			map,
			course,
			mode: record.mode_name,
			player: CompactPlayer {
				name: player.name.clone(),
				steam_id64: (player.id as u64 + MAGIC_STEAM_ID_OFFSET).to_string(),
			},
			server: server.name.clone(),
			time: record.time,
			teleports: record.teleports,
			created_on: record.created_on.to_string(),
		});
	}

	Ok(records)
}

const ONLY_MAP: &str = r#"
		  player.name AS player_name,
		  player.id AS player_id,
		  server.name AS server_name
"#;
const ONLY_MAP_JOINS: &str = r#"
		JOIN modes AS mode ON mode.id = record.mode_id
		JOIN servers AS server ON server.id = record.server_id
"#;
async fn only_map(
	map: String,
	mut filter: String,
	limit: u32,
	pool: &Pool<MySql>,
) -> Eyre<Vec<FancyRecord>> {
	let (maybe_map, f) = match map.parse::<MapIdentifier>()? {
		MapIdentifier::ID(id) => (None, format!("course.map_id = {id}")),
		ident @ MapIdentifier::Name(_) => {
			let map = get_map(&ident, pool).await?;
			let f = format!("course.map_id = {}", map.id);
			(Some(map), f)
		}
	};

	filter.push_str(&format!(
		r#"
		AND {f}
		"#,
	));

	filter = filter.replacen("AND", "WHERE", 1);

	let query = format!(
		r#"
		{BASE_QUERY}
		{ONLY_MAP}
		{BASE_JOINS}
		{ONLY_MAP_JOINS}
		{filter}
		ORDER BY record.created_on
		LIMIT {limit}
		"#
	);

	let map = if let Some(map) = maybe_map { map } else { get_map(&map.parse()?, pool).await? };

	let mut records = Vec::new();
	for record in sqlx::query_as::<_, RecordWithoutMap>(&query)
		.fetch_all(pool)
		.await?
	{
		let course = get_course(record.course_id, pool).await?;
		let steam_id64 = record.player_id as u64 + MAGIC_STEAM_ID_OFFSET;
		let player = get_player_raw(PlayerIdentifier::SteamID64(steam_id64), pool).await?;

		records.push(FancyRecord {
			id: record.id,
			map: map.clone(),
			course,
			mode: record.mode_name,
			player: CompactPlayer {
				name: player.name,
				steam_id64: steam_id64.to_string(),
			},
			server: record.server_name,
			time: record.time,
			teleports: record.teleports,
			created_on: record.created_on.to_string(),
		});
	}

	Ok(records)
}

const MAP_SERVER: &str = r#"
		  player.name AS player_name,
		  player.id AS player_id,
		  record.server_id AS server_id
"#;
const MAP_SERVER_JOINS: &str = r#"
		JOIN players AS player ON player.id = record.player_id
"#;
async fn map_server(
	map: String,
	server: String,
	mut filter: String,
	limit: u32,
	pool: &Pool<MySql>,
) -> Eyre<Vec<FancyRecord>> {
	let (maybe_map, f) = match map.parse::<MapIdentifier>()? {
		MapIdentifier::ID(id) => (None, format!("course.map_id = {id}")),
		ident @ MapIdentifier::Name(_) => {
			let map = get_map(&ident, pool).await?;
			let f = format!("course.map_id = {}", map.id);
			(Some(map), f)
		}
	};

	filter.push_str(&format!(
		r#"
		AND {f}
		"#,
	));

	let (maybe_server, f) = match server.parse::<u16>() {
		Ok(server_id) => (None, format!("record.server_id = {server_id}")),
		_ => {
			let server = get_servers(
				QueryInput::Filter(format!(
					r#"
					WHERE {} {}
					"#,
					if server.parse::<u16>().is_ok() { "id =" } else { "name LIKE" },
					if let Ok(id) = server.parse::<u16>() {
						format!("{id}")
					} else {
						format!(r#""%{server}%""#)
					}
				)),
				pool,
			)
			.await?
			.remove(0);

			let f = format!("record.server_id = {}", server.id);

			(Some(server), f)
		}
	};

	filter.push_str(&format!(
		r#"
		AND {f}
		"#,
	));

	filter = filter.replacen("AND", "WHERE", 1);

	let query = format!(
		r#"
		{BASE_QUERY}
		{MAP_SERVER}
		{BASE_JOINS}
		{MAP_SERVER_JOINS}
		{filter}
		ORDER BY record.created_on
		LIMIT {limit}
		"#
	);

	let map = if let Some(map) = maybe_map { map } else { get_map(&map.parse()?, pool).await? };
	let server = if let Some(server) = maybe_server {
		server
	} else {
		get_servers(
			QueryInput::Filter(format!(
				r#"
				WHERE {} {}
				"#,
				if server.parse::<u16>().is_ok() { "id =" } else { "name LIKE" },
				if let Ok(id) = server.parse::<u16>() {
					format!("{id}")
				} else {
					format!(r#""%{server}%""#)
				}
			)),
			pool,
		)
		.await?
		.remove(0)
	};

	let mut records = Vec::new();
	for record in sqlx::query_as::<_, RecordWithoutMapOrServer>(&query)
		.fetch_all(pool)
		.await?
	{
		let course = get_course(record.course_id, pool).await?;
		let steam_id64 = record.player_id as u64 + MAGIC_STEAM_ID_OFFSET;
		let player = get_player_raw(PlayerIdentifier::SteamID64(steam_id64), pool).await?;

		records.push(FancyRecord {
			id: record.id,
			map: map.clone(),
			course,
			mode: record.mode_name,
			player: CompactPlayer {
				name: player.name,
				steam_id64: steam_id64.to_string(),
			},
			server: server.name.clone(),
			time: record.time,
			teleports: record.teleports,
			created_on: record.created_on.to_string(),
		});
	}

	Ok(records)
}

const MAP_PLAYER: &str = r#"
		  server.name AS server_name
"#;
const MAP_PLAYER_JOINS: &str = r#"
		JOIN servers AS server ON server.id = record.server_id
"#;
async fn map_player(
	map: String,
	player: String,
	mut filter: String,
	limit: u32,
	pool: &Pool<MySql>,
) -> Eyre<Vec<FancyRecord>> {
	let (maybe_map, f) = match map.parse::<MapIdentifier>()? {
		MapIdentifier::ID(id) => (None, format!("course.map_id = {id}")),
		ident @ MapIdentifier::Name(_) => {
			let map = get_map(&ident, pool).await?;
			let f = format!("course.map_id = {}", map.id);
			(Some(map), f)
		}
	};

	filter.push_str(&format!(
		r#"
		AND {f}
		"#,
	));

	let (maybe_player, f) = match player.parse::<PlayerIdentifier>()? {
		ident @ PlayerIdentifier::Name(_) => {
			let player = get_player_raw(ident, pool).await?;
			let f = format!(r#"record.player_id = {}"#, player.id);
			(Some(player), f)
		}
		PlayerIdentifier::SteamID(steam_id) => {
			let account_id =
				steam_id_to_account_id(&steam_id.to_string()).ok_or(eyre!("Invalid SteamID"))?;
			(None, format!(r#"record.player_id = {account_id}"#))
		}
		PlayerIdentifier::SteamID64(steam_id64) => {
			let account_id = steam_id64_to_account_id(steam_id64)?;
			(None, format!(r#"record.player_id = {account_id}"#))
		}
	};

	filter.push_str(&format!(
		r#"
		AND {f}
		"#,
	));

	filter = filter.replacen("AND", "WHERE", 1);

	let query = format!(
		r#"
		{BASE_QUERY}
		{MAP_PLAYER}
		{BASE_JOINS}
		{MAP_PLAYER_JOINS}
		{filter}
		ORDER BY record.created_on
		LIMIT {limit}
		"#
	);

	let map = if let Some(map) = maybe_map { map } else { get_map(&map.parse()?, pool).await? };
	let player = if let Some(player) = maybe_player {
		player
	} else {
		get_player_raw(player.parse()?, pool).await?
	};
	let steam_id64 = player.id as u64 + MAGIC_STEAM_ID_OFFSET;

	let mut records = Vec::new();
	for record in sqlx::query_as::<_, RecordWithoutMapOrPlayer>(&query)
		.fetch_all(pool)
		.await?
	{
		let course = get_course(record.course_id, pool).await?;

		records.push(FancyRecord {
			id: record.id,
			map: map.clone(),
			course,
			mode: record.mode_name,
			player: CompactPlayer {
				name: player.name.clone(),
				steam_id64: steam_id64.to_string(),
			},
			server: record.server_name,
			time: record.time,
			teleports: record.teleports,
			created_on: record.created_on.to_string(),
		});
	}

	Ok(records)
}

async fn all_params(
	map: String,
	player: String,
	server: String,
	mut filter: String,
	limit: u32,
	pool: &Pool<MySql>,
) -> Eyre<Vec<FancyRecord>> {
	let (maybe_map, f) = match map.parse::<MapIdentifier>()? {
		MapIdentifier::ID(id) => (None, format!("course.map_id = {id}")),
		ident @ MapIdentifier::Name(_) => {
			let map = get_map(&ident, pool).await?;
			let f = format!("course.map_id = {}", map.id);
			(Some(map), f)
		}
	};

	filter.push_str(&format!(
		r#"
		AND {f}
		"#,
	));

	let (maybe_player, f) = match player.parse::<PlayerIdentifier>()? {
		ident @ PlayerIdentifier::Name(_) => {
			let player = get_player_raw(ident, pool).await?;
			let f = format!(r#"record.player_id = {}"#, player.id);
			(Some(player), f)
		}
		PlayerIdentifier::SteamID(steam_id) => {
			let account_id =
				steam_id_to_account_id(&steam_id.to_string()).ok_or(eyre!("Invalid SteamID"))?;
			(None, format!(r#"record.player_id = {account_id}"#))
		}
		PlayerIdentifier::SteamID64(steam_id64) => {
			let account_id = steam_id64_to_account_id(steam_id64)?;
			(None, format!(r#"record.player_id = {account_id}"#))
		}
	};

	filter.push_str(&format!(
		r#"
		AND {f}
		"#,
	));

	let (maybe_server, f) = match server.parse::<u16>() {
		Ok(server_id) => (None, format!("record.server_id = {server_id}")),
		_ => {
			let server = get_servers(
				QueryInput::Filter(format!(
					r#"
					WHERE {} {}
					"#,
					if server.parse::<u16>().is_ok() { "id =" } else { "name LIKE" },
					if let Ok(id) = server.parse::<u16>() {
						format!("{id}")
					} else {
						format!(r#""%{server}%""#)
					}
				)),
				pool,
			)
			.await?
			.remove(0);

			let f = format!("record.server_id = {}", server.id);

			(Some(server), f)
		}
	};

	filter.push_str(&format!(
		r#"
		AND {f}
		"#,
	));

	filter = filter.replacen("AND", "WHERE", 1);

	let query = format!(
		r#"
		{}
		{BASE_JOINS}
		{filter}
		ORDER BY record.created_on
		LIMIT {limit}
		"#,
		BASE_QUERY.strip_suffix(",\n").unwrap()
	);

	let map = if let Some(map) = maybe_map { map } else { get_map(&map.parse()?, pool).await? };
	let player = if let Some(player) = maybe_player {
		player
	} else {
		get_player_raw(player.parse()?, pool).await?
	};
	let server = if let Some(server) = maybe_server {
		server
	} else {
		get_servers(
			QueryInput::Filter(format!(
				r#"
				WHERE {} {}
				"#,
				if server.parse::<u16>().is_ok() { "id =" } else { "name LIKE" },
				if let Ok(id) = server.parse::<u16>() {
					format!("{id}")
				} else {
					format!(r#""%{server}%""#)
				}
			)),
			pool,
		)
		.await?
		.remove(0)
	};

	let mut records = Vec::new();
	for record in sqlx::query_as::<_, RecordWithoutMapOrPlayerOrServer>(&query)
		.fetch_all(pool)
		.await?
	{
		let course = get_course(record.course_id, pool).await?;

		records.push(FancyRecord {
			id: record.id,
			map: map.clone(),
			course,
			mode: record.mode_name,
			player: CompactPlayer {
				name: player.name.clone(),
				steam_id64: (player.id as u64 + MAGIC_STEAM_ID_OFFSET).to_string(),
			},
			server: server.name.clone(),
			time: record.time,
			teleports: record.teleports,
			created_on: record.created_on.to_string(),
		});
	}

	Ok(records)
}
