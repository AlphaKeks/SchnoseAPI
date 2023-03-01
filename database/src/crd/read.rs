use {
	crate::schemas::*,
	color_eyre::{eyre::eyre, Result as Eyre},
	gokz_rs::prelude::*,
	log::debug,
	sqlx::{MySql, Pool},
};

pub async fn get_mode(mode: Mode, pool: &Pool<MySql>) -> Eyre<ModeRow> {
	debug!("Mode: {mode:?}");
	Ok(sqlx::query_as::<_, ModeRow>(&format!(
		r#"
		SELECT * FROM modes
		WHERE id = {}
		"#,
		mode as u8
	))
	.fetch_one(pool)
	.await?)
}

pub async fn get_modes(pool: &Pool<MySql>) -> Eyre<Vec<ModeRow>> {
	Ok(sqlx::query_as::<_, ModeRow>("SELECT * FROM modes")
		.fetch_all(pool)
		.await?)
}

pub async fn get_player(player: PlayerIdentifier, pool: &Pool<MySql>) -> Eyre<PlayerRow> {
	debug!("Player: {player:?}");
	let filter = match player {
		PlayerIdentifier::Name(player_name) => {
			format!(r#"name LIKE "{player_name}%""#)
		}
		PlayerIdentifier::SteamID(steam_id) => {
			let account_id =
				steam_id_to_account_id(&steam_id.to_string()).ok_or(eyre!("Bad SteamID"))?;
			format!("id = {account_id}")
		}
		PlayerIdentifier::SteamID64(steam_id64) => {
			let account_id = steam_id64_to_account_id(steam_id64)?;
			format!("id = {account_id}")
		}
	};

	Ok(sqlx::query_as::<_, PlayerRow>(&format!(
		r#"
		SELECT * FROM players
		WHERE {filter}
		"#
	))
	.fetch_one(pool)
	.await?)
}

pub async fn get_server(server: String, pool: &Pool<MySql>) -> Eyre<ServerRow> {
	debug!("Server: {server:?}");
	let filter = if let Ok(server_id) = server.parse::<u16>() {
		format!("id = {server_id}")
	} else {
		format!(r#"name LIKE "%{server}%""#)
	};

	Ok(sqlx::query_as::<_, ServerRow>(&format!(
		r#"
		SELECT * FROM servers
		WHERE {filter}
		LIMIT 1
		"#
	))
	.fetch_one(pool)
	.await?)
}

pub async fn get_servers(pool: &Pool<MySql>) -> Eyre<Vec<ServerRow>> {
	Ok(sqlx::query_as::<_, ServerRow>("SELECT * FROM servers")
		.fetch_all(pool)
		.await?)
}

pub async fn get_map(map: MapIdentifier, pool: &Pool<MySql>) -> Eyre<MapRow> {
	debug!("Map: {map:?}");
	let filter = match map {
		MapIdentifier::ID(map_id) => format!("id = {map_id}"),
		MapIdentifier::Name(map_name) => format!(r#"name LIKE "%{map_name}%""#),
	};

	Ok(sqlx::query_as::<_, MapRow>(&format!(
		r#"
		SELECT * FROM maps
		WHERE {filter}
		LIMIT 1
		"#
	))
	.fetch_one(pool)
	.await?)
}

pub async fn get_maps(pool: &Pool<MySql>) -> Eyre<Vec<MapRow>> {
	Ok(sqlx::query_as::<_, MapRow>("SELECT * FROM maps")
		.fetch_all(pool)
		.await?)
}

pub async fn get_course(course_id: u32, pool: &Pool<MySql>) -> Eyre<CourseRow> {
	debug!("Course: {course_id:?}");
	Ok(sqlx::query_as::<_, CourseRow>(&format!(
		r#"
		SELECT * FROM courses
		WHERE id = {course_id}
		"#
	))
	.fetch_one(pool)
	.await?)
}

pub async fn get_courses(map_id: u16, pool: &Pool<MySql>) -> Eyre<Vec<CourseRow>> {
	Ok(sqlx::query_as::<_, CourseRow>(&format!(
		r#"
		SELECT * FROM courses
		WHERE map_id = {map_id}
		"#
	))
	.fetch_all(pool)
	.await?)
}

pub async fn get_record(record_id: u32, pool: &Pool<MySql>) -> Eyre<RecordRow> {
	debug!("Record: {record_id:?}");
	Ok(sqlx::query_as::<_, RecordRow>(&format!(
		r#"
		SELECT * FROM records
		WHERE id = {record_id}
		"#
	))
	.fetch_one(pool)
	.await?)
}
