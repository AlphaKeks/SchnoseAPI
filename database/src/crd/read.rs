use {
	crate::schemas::*,
	gokz_rs::prelude::*,
	log::debug,
	sqlx::{Error as SQLError, MySql, Pool, QueryBuilder},
};

type Result<T> = std::result::Result<T, SQLError>;

pub async fn get_mode(mode: Mode, pool: &Pool<MySql>) -> Result<ModeRow> {
	debug!("Mode: {mode:?}");
	let mut query = QueryBuilder::new(
		r#"
		SELECT * FROM modes
		WHERE id =
		"#,
	);

	query.push_bind(mode as u8);

	query
		.build_query_as()
		.fetch_one(pool)
		.await
}

pub async fn get_modes(pool: &Pool<MySql>) -> Result<Vec<ModeRow>> {
	sqlx::query_as::<_, ModeRow>("SELECT * FROM modes")
		.fetch_all(pool)
		.await
}

pub async fn get_player(player: PlayerIdentifier, pool: &Pool<MySql>) -> Result<PlayerRow> {
	debug!("Player: {player:?}");
	let mut query = QueryBuilder::new(
		r#"
		SELECT * FROM players
		WHERE
		"#,
	);

	match player {
		PlayerIdentifier::Name(player_name) => {
			query
				.push("name LIKE ")
				.push_bind(format!(r#"{player_name}%"#));
		}
		PlayerIdentifier::SteamID(steam_id) => {
			let account_id =
				steam_id_to_account_id(&steam_id.to_string()).ok_or(SQLError::RowNotFound)?;
			query
				.push("id = ")
				.push_bind(account_id);
		}
		PlayerIdentifier::SteamID64(steam_id64) => {
			let account_id =
				steam_id64_to_account_id(steam_id64).map_err(|_| SQLError::RowNotFound)?;
			query
				.push("id = ")
				.push_bind(account_id);
		}
	};

	query
		.build_query_as()
		.fetch_one(pool)
		.await
}

pub async fn get_server(server: String, pool: &Pool<MySql>) -> Result<ServerRow> {
	debug!("Server: {server:?}");
	let mut query = QueryBuilder::new(
		r#"
		SELECT * FROM servers
		WHERE
		"#,
	);

	if let Ok(server_id) = server.parse::<u16>() {
		query.push("id = ").push_bind(server_id);
	} else {
		query
			.push("name LIKE ")
			.push_bind(format!(r#"%{server}%"#));
	};

	query.push(" LIMIT 1 ");

	query
		.build_query_as()
		.fetch_one(pool)
		.await
}

pub async fn get_servers(pool: &Pool<MySql>) -> Result<Vec<ServerRow>> {
	sqlx::query_as::<_, ServerRow>("SELECT * FROM servers")
		.fetch_all(pool)
		.await
}

pub async fn get_map(map: MapIdentifier, pool: &Pool<MySql>) -> Result<MapRow> {
	debug!("Map: {map:?}");
	let mut query = QueryBuilder::new(
		r#"
		SELECT * FROM maps
		WHERE
		"#,
	);

	match map {
		MapIdentifier::ID(map_id) => query.push("id = ").push_bind(map_id),
		MapIdentifier::Name(map_name) => query
			.push("name LIKE ")
			.push_bind(format!(r#"%{map_name}%"#)),
	};

	query.push(" LIMIT 1 ");

	query
		.build_query_as()
		.fetch_one(pool)
		.await
}

pub async fn get_maps(pool: &Pool<MySql>) -> Result<Vec<MapRow>> {
	sqlx::query_as::<_, MapRow>("SELECT * FROM maps")
		.fetch_all(pool)
		.await
}

pub async fn get_course(course_id: u16, pool: &Pool<MySql>) -> Result<CourseRow> {
	debug!("Course: {course_id:?}");
	sqlx::query_as::<_, CourseRow>(&format!(
		r#"
		SELECT * FROM courses
		WHERE id = {course_id}
		"#
	))
	.fetch_one(pool)
	.await
}

pub async fn get_courses(map_id: u16, pool: &Pool<MySql>) -> Result<Vec<CourseRow>> {
	sqlx::query_as::<_, CourseRow>(&format!(
		r#"
		SELECT * FROM courses
		WHERE map_id = {map_id}
		"#
	))
	.fetch_all(pool)
	.await
}

pub async fn get_record(record_id: u32, pool: &Pool<MySql>) -> Result<RecordRow> {
	debug!("Record: {record_id:?}");
	sqlx::query_as::<_, RecordRow>(&format!(
		r#"
		SELECT * FROM records
		WHERE id = {record_id}
		"#
	))
	.fetch_one(pool)
	.await
}
