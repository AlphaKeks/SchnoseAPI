use {
	crate::schemas::{self, steam_id64_to_account_id, steam_id_to_account_id},
	color_eyre::{eyre::eyre, Result as Eyre},
	gokz_rs::prelude::*,
	log::debug,
	sqlx::{MySql, Pool},
};

pub enum QueryInput {
	Query(String),
	Limit(usize),
	Filter(String),
}

pub async fn get_modes(pool: &Pool<MySql>) -> Eyre<Vec<schemas::Mode>> {
	let query = String::from(r#"SELECT * FROM modes"#);
	debug!("[get_modes] Query: {query}");

	Ok(sqlx::query_as::<_, schemas::raw::ModeRow>(&query)
		.fetch_all(pool)
		.await?
		.into_iter()
		.map(|row| {
			debug!("Parsing row {row:?}");
			let mode = Mode::from(&row);
			schemas::Mode {
				id: mode as u8,
				name: mode.api(),
				name_short: mode.short(),
				name_long: mode.to_string(),
				created_on: row.created_on.to_string(),
			}
		})
		.collect())
}

pub async fn get_mode(mode: Mode, pool: &Pool<MySql>) -> Eyre<schemas::Mode> {
	let query = format!(r#"SELECT * FROM modes WHERE id = {}"#, mode as u8);
	debug!("[get_mode] Query: {query}");

	Ok(sqlx::query_as::<_, schemas::raw::ModeRow>(&query)
		.fetch_one(pool)
		.await
		.map(|row| {
			debug!("Parsing row {row:?}");
			let mode = Mode::from(&row);
			schemas::Mode {
				id: mode as u8,
				name: mode.api(),
				name_short: mode.short(),
				name_long: mode.to_string(),
				created_on: row.created_on.to_string(),
			}
		})?)
}

const PLAYER_QUERY: &str = r#"
SELECT
  p.id AS account_id,
  p.name AS name,
  p.is_banned AS is_banned,
  COUNT(*) AS total_records,
  SUM(r.mode_id = 200 AND r.teleports > 0) AS kzt_tp_records,
  SUM(r.mode_id = 200 AND r.teleports = 0) AS kzt_pro_records,
  SUM(r.mode_id = 201 AND r.teleports > 0) AS skz_tp_records,
  SUM(r.mode_id = 201 AND r.teleports = 0) AS skz_pro_records,
  SUM(r.mode_id = 202 AND r.teleports > 0) AS vnl_tp_records,
  SUM(r.mode_id = 202 AND r.teleports = 0) AS vnl_pro_records
FROM players as p
JOIN records AS r ON p.id = r.player_id
GROUP BY p.id
ORDER BY p.id
"#;

pub async fn get_players(input: QueryInput, pool: &Pool<MySql>) -> Eyre<Vec<schemas::FancyPlayer>> {
	let query = match input {
		QueryInput::Query(query) => query,
		QueryInput::Limit(limit) => format!("{PLAYER_QUERY}\nLIMIT {limit}"),
		QueryInput::Filter(filter) => {
			format!("SELECT * FROM ({PLAYER_QUERY}) AS player {filter}")
		}
	};
	debug!("[get_players] Query: {query}");

	let players = sqlx::query_as::<_, schemas::Player>(&query)
		.fetch_all(pool)
		.await?
		.into_iter()
		.filter_map(|row| {
			debug!("Parsing row {row:?}");
			schemas::FancyPlayer::try_from(row).ok()
		})
		.collect::<Vec<_>>();

	if players.is_empty() {
		Err(eyre!("NO PLAYERS FOUND"))
	} else {
		Ok(players)
	}
}

pub async fn get_player_raw(
	player_ident: PlayerIdentifier,
	pool: &Pool<MySql>,
) -> Eyre<schemas::raw::PlayerRow> {
	let query = format!(
		r#"
		SELECT * FROM players
		WHERE {}
		"#,
		match player_ident {
			PlayerIdentifier::Name(name) => format!(r#"name LIKE "%{name}%""#),
			PlayerIdentifier::SteamID(steam_id) => {
				let account_id = steam_id_to_account_id(&steam_id.to_string())
					.ok_or(eyre!("Invalid SteamID"))?;
				format!(r#"id = {account_id}"#)
			}
			PlayerIdentifier::SteamID64(steam_id64) => {
				let account_id = steam_id64_to_account_id(steam_id64)?;
				format!(r#"id = {account_id}"#)
			}
		}
	);

	Ok(sqlx::query_as::<_, schemas::raw::PlayerRow>(&query)
		.fetch_one(pool)
		.await?)
}

const SERVER_QUERY: &str = r#"
SELECT
  s.id AS id,
  s.name AS name,
  o.id AS owner_id,
  o.name AS owner_name,
  o.is_banned AS owner_is_banned,
  a.id AS approved_by_id,
  a.name AS approved_by_name,
  a.is_banned AS approved_by_is_banned
FROM servers AS s
JOIN players AS o ON o.id = s.owned_by
JOIN players AS a ON a.id = s.approved_by
"#;

pub async fn get_servers(input: QueryInput, pool: &Pool<MySql>) -> Eyre<Vec<schemas::FancyServer>> {
	let query = match input {
		QueryInput::Query(query) => query,
		QueryInput::Limit(limit) => format!("{SERVER_QUERY}\nLIMIT {limit}"),
		QueryInput::Filter(filter) => {
			format!("SELECT * FROM ({SERVER_QUERY}) AS server {filter}")
		}
	};

	let servers = sqlx::query_as::<_, schemas::Server>(&query)
		.fetch_all(pool)
		.await?
		.into_iter()
		.filter_map(|row| {
			debug!("Parsing row {row:?}");
			schemas::FancyServer::try_from(row).ok()
		})
		.collect::<Vec<_>>();

	if servers.is_empty() {
		Err(eyre!("NO SERVERS FOUND"))
	} else {
		Ok(servers)
	}
}

const MAP_QUERY: &str = r#"
SELECT
  m.id AS id,
  m.name AS name,
  co.kzt_difficulty AS tier,
  m.courses AS courses,
  m.validated AS validated,
  m.filesize AS filesize,
  cr.id AS creator_id,
  cr.name AS creator_name,
  cr.is_banned AS creator_is_banned,
  ap.id AS approver_id,
  ap.name AS approver_name,
  ap.is_banned AS approver_is_banned,
  m.created_on AS created_on,
  m.updated_on AS updated_on
FROM maps AS m
JOIN courses AS co ON co.map_id = m.id
JOIN players AS cr ON cr.id = m.created_by
JOIN players AS ap ON ap.id = m.approved_by
"#;

pub async fn get_maps(input: QueryInput, pool: &Pool<MySql>) -> Eyre<Vec<schemas::FancyMap>> {
	let query = match input {
		QueryInput::Query(query) => query,
		QueryInput::Limit(limit) => format!("{MAP_QUERY}\nLIMIT {limit}"),
		QueryInput::Filter(filter) => {
			format!("SELECT * FROM ({MAP_QUERY}) AS map {filter}")
		}
	};
	debug!("[get_maps] Query: {query}");

	let maps = sqlx::query_as::<_, schemas::Map>(&query)
		.fetch_all(pool)
		.await?
		.into_iter()
		.filter_map(|row| {
			debug!("Parsing row {row:?}");
			schemas::FancyMap::try_from(row).ok()
		})
		.collect::<Vec<_>>();

	if maps.is_empty() {
		Err(eyre!("NO MAPS FOUND"))
	} else {
		Ok(maps)
	}
}

pub async fn get_map(map: &MapIdentifier, pool: &Pool<MySql>) -> Eyre<schemas::FancyMap> {
	let filter = match map {
		MapIdentifier::ID(map_id) => format!("WHERE map.id = {map_id}"),
		MapIdentifier::Name(map_name) => format!(r#"WHERE map.name LIKE "%{map_name}%""#),
	};

	Ok(get_maps(QueryInput::Filter(filter), pool)
		.await?
		.remove(0))
}

pub async fn get_course(course_id: u32, pool: &Pool<MySql>) -> Eyre<schemas::Course> {
	let course = sqlx::query_as::<_, schemas::raw::CourseRow>(&format!(
		r#"
		SELECT * FROM courses
		WHERE id = {course_id}
		"#
	))
	.fetch_one(pool)
	.await?;

	Ok(schemas::Course {
		id: course.id,
		map: get_map(&MapIdentifier::ID(course.map_id as i32), pool).await?,
		stage: course.stage,
		kzt: course.kzt,
		kzt_difficulty: Tier::try_from(course.kzt_difficulty)? as u8,
		skz: course.skz,
		skz_difficulty: Tier::try_from(course.skz_difficulty)? as u8,
		vnl: course.vnl,
		vnl_difficulty: Tier::try_from(course.vnl_difficulty)? as u8,
	})
}
