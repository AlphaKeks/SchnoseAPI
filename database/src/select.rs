use {
	crate::schemas::*,
	gokz_rs::{MapIdentifier, Mode, PlayerIdentifier, ServerIdentifier},
	log::{debug, info},
	sqlx::{MySql, Pool, QueryBuilder, Result},
};

pub async fn get_mode(mode: Mode, pool: &Pool<MySql>) -> Result<ModeRow> {
	info!("Fetching {mode:?} from DB.");
	let mut query = QueryBuilder::new("SELECT * FROM modes WHERE id = ");
	query.push_bind(mode as u8);
	debug!("Runnig query:\n{}", query.sql());
	query
		.build_query_as()
		.fetch_one(pool)
		.await
}

pub async fn get_modes(limit: Option<u32>, pool: &Pool<MySql>) -> Result<Vec<ModeRow>> {
	info!("Fetching {limit:?} modes from DB.");
	let mut query = QueryBuilder::new("SELECT * FROM modes LIMIT ");
	query.push_bind(limit.unwrap_or(u32::MAX));
	debug!("Runnig query:\n{}", query.sql());
	query
		.build_query_as()
		.fetch_all(pool)
		.await
}

pub async fn get_player(player: PlayerIdentifier, pool: &Pool<MySql>) -> Result<PlayerRow> {
	info!("Fetching {player:?} from DB.");
	let mut query = QueryBuilder::new("SELECT * FROM players WHERE ");

	match player {
		PlayerIdentifier::SteamID(steam_id) => query
			.push("id = ")
			.push_bind(steam_id.as_id32()),
		PlayerIdentifier::Name(player_name) => query
			.push(r#"name LIKE "#)
			.push_bind(format!("%{player_name}%")),
	};

	debug!("Runnig query:\n{}", query.sql());

	query
		.build_query_as()
		.fetch_one(pool)
		.await
}

pub async fn get_players(limit: Option<u32>, pool: &Pool<MySql>) -> Result<Vec<PlayerRow>> {
	info!("Fetching {limit:?} players from DB.");
	Ok(sqlx::query!("SELECT * FROM players LIMIT ?", limit.unwrap_or(u32::MAX))
		.fetch_all(pool)
		.await?
		.into_iter()
		.map(|row| PlayerRow {
			id: row.id,
			name: row.name,
			is_banned: row.is_banned == 1,
		})
		.collect())
}

pub async fn get_course(course_id: u32, pool: &Pool<MySql>) -> Result<CourseRow> {
	info!("Fetching course #{course_id} from DB.");
	sqlx::query!("SELECT * FROM courses WHERE id = ?", course_id)
		.fetch_one(pool)
		.await
		.map(|row| CourseRow {
			id: row.id,
			map_id: row.map_id,
			stage: row.stage,
			kzt: row.kzt == 1,
			kzt_difficulty: row.kzt_difficulty,
			skz: row.skz == 1,
			skz_difficulty: row.skz_difficulty,
			vnl: row.vnl == 1,
			vnl_difficulty: row.vnl_difficulty,
		})
}

pub async fn get_course_by_map(map: MapIdentifier, pool: &Pool<MySql>) -> Result<CourseRow> {
	info!("Fetching course of map #{map:?} from DB.");
	let mut query = QueryBuilder::new("SELECT * FROM courses ");

	match map {
		MapIdentifier::Name(map_name) => query
			.push(r#"JOIN maps ON maps.name LIKE "%"#)
			.push_bind(map_name)
			.push(r#"%""#),
		MapIdentifier::ID(map_id) => query
			.push("WHERE map_id = ")
			.push_bind(map_id),
	};

	debug!("Runnig query:\n{}", query.sql());

	query
		.build_query_as()
		.fetch_one(pool)
		.await
}

pub async fn get_courses(limit: Option<u32>, pool: &Pool<MySql>) -> Result<Vec<CourseRow>> {
	info!("Fetching {limit:?} courses from DB.");
	Ok(sqlx::query!("SELECT * FROM courses LIMIT ?", limit.unwrap_or(u32::MAX))
		.fetch_all(pool)
		.await?
		.into_iter()
		.map(|row| CourseRow {
			id: row.id,
			map_id: row.map_id,
			stage: row.stage,
			kzt: row.kzt == 1,
			kzt_difficulty: row.kzt_difficulty,
			skz: row.skz == 1,
			skz_difficulty: row.skz_difficulty,
			vnl: row.vnl == 1,
			vnl_difficulty: row.vnl_difficulty,
		})
		.collect())
}

pub async fn get_map(map: MapIdentifier, pool: &Pool<MySql>) -> Result<MapRow> {
	info!("Fetching {map:?} from DB.");
	let mut query = QueryBuilder::new("SELECT * FROM maps WHERE ");

	match map {
		MapIdentifier::Name(map_name) => query
			.push(r#"name LIKE "%"#)
			.push_bind(map_name)
			.push(r#"%""#),
		MapIdentifier::ID(map_id) => query.push("id = ").push_bind(map_id),
	};

	debug!("Runnig query:\n{}", query.sql());

	query
		.build_query_as()
		.fetch_one(pool)
		.await
}

pub async fn get_maps(limit: Option<u32>, pool: &Pool<MySql>) -> Result<Vec<MapRow>> {
	info!("Fetching {limit:?} maps from DB.");
	let mut query = QueryBuilder::new("SELECT * FROM maps LIMIT ?");
	query.push_bind(limit.unwrap_or(u32::MAX));
	debug!("Runnig query:\n{}", query.sql());
	query
		.build_query_as::<MapRow>()
		.fetch_all(pool)
		.await
}

pub async fn get_server(server: ServerIdentifier, pool: &Pool<MySql>) -> Result<ServerRow> {
	info!("Fetching {server:?} from DB.");
	let mut query = QueryBuilder::new("SELECT * FROM servers WHERE ");

	match server {
		ServerIdentifier::Name(server_name) => query
			.push(r#"name LIKE "%"#)
			.push_bind(server_name)
			.push(r#"%""#),
		ServerIdentifier::ID(server_id) => query.push("id = ").push_bind(server_id),
	};

	debug!("Runnig query:\n{}", query.sql());

	query
		.build_query_as()
		.fetch_one(pool)
		.await
}

pub async fn get_servers(limit: Option<u32>, pool: &Pool<MySql>) -> Result<Vec<ServerRow>> {
	info!("Fetching {limit:?} servers from DB.");
	sqlx::query_as!(ServerRow, "SELECT * FROM servers LIMIT ?", limit.unwrap_or(u32::MAX))
		.fetch_all(pool)
		.await
}

pub async fn get_record(record_id: u32, pool: &Pool<MySql>) -> Result<RecordRow> {
	info!("Fetching record #{record_id} from DB.");
	let mut query = QueryBuilder::new("SELECT * FROM records WHERE id = ");
	query.push_bind(record_id);
	debug!("Runnig query:\n{}", query.sql());
	query
		.build_query_as()
		.fetch_one(pool)
		.await
}
