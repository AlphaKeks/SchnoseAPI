use {
	chrono::{DateTime, Utc},
	color_eyre::Result as Eyre,
	sqlx::{MySql, Pool, QueryBuilder},
};

pub type ModeData = (u8, String, DateTime<Utc>);
pub async fn insert_modes(modes: &[ModeData], pool: &Pool<MySql>) -> Eyre<()> {
	let mut transaction = pool.begin().await?;

	// for (id, name, created_on) in modes {
	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO modes
		  (id, name, created_on)
		"#,
	);
	query
		.push_values(modes, |mut query, (id, name, created_on)| {
			query
				.push_bind(id)
				.push_bind(name)
				.push_bind(created_on);
		})
		.build()
		.execute(&mut transaction)
		.await?;

	transaction.commit().await?;

	Ok(())
}

pub type PlayerData = (u32, String, u8);
pub async fn insert_players(players: &[PlayerData], pool: &Pool<MySql>) -> Eyre<()> {
	let mut transaction = pool.begin().await?;

	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO players
		  (id, name, is_banned)
		"#,
	);
	query
		.push_values(players, |mut query, (id, name, is_banned)| {
			query
				.push_bind(id)
				.push_bind(name)
				.push_bind(is_banned);
		})
		.build()
		.execute(&mut transaction)
		.await?;

	transaction.commit().await?;

	Ok(())
}

pub type ServerData = (u16, String, u32, u32);
pub async fn insert_servers(servers: &[ServerData], pool: &Pool<MySql>) -> Eyre<()> {
	let mut transaction = pool.begin().await?;

	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO servers
		  (id, name, owned_by, approved_by)
		"#,
	);
	query
		.push_values(servers, |mut query, (id, name, owned_by, approved_by)| {
			query
				.push_bind(id)
				.push_bind(name)
				.push_bind(owned_by)
				.push_bind(approved_by);
		})
		.build()
		.execute(&mut transaction)
		.await?;

	transaction.commit().await?;

	Ok(())
}

pub type MapData = (u16, String, u8, bool, u64, u32, u32, DateTime<Utc>, DateTime<Utc>);
pub async fn insert_maps(maps: &[MapData], pool: &Pool<MySql>) -> Eyre<()> {
	let mut transaction = pool.begin().await?;

	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO maps
		  (id, name, courses, validated, filesize, created_by, approved_by, created_on, updated_on)
		"#,
	);
	query
		.push_values(
			maps,
			|mut query,
			 (
				id,
				name,
				courses,
				validated,
				filesize,
				created_by,
				approved_by,
				created_on,
				updated_on,
			)| {
				query
					.push_bind(id)
					.push_bind(name)
					.push_bind(courses)
					.push_bind(validated)
					.push_bind(filesize)
					.push_bind(created_by)
					.push_bind(approved_by)
					.push_bind(created_on)
					.push_bind(updated_on);
			},
		)
		.build()
		.execute(&mut transaction)
		.await?;

	transaction.commit().await?;

	Ok(())
}

pub type CourseData = (u32, u16, u8, bool, u8, bool, u8, bool, u8);
pub async fn insert_courses(courses: &[CourseData], pool: &Pool<MySql>) -> Eyre<()> {
	let mut transaction = pool.begin().await?;

	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO courses
		  (id, map_id, stage, kzt, kzt_difficulty, skz, skz_difficutly, vnl, vnl_difficulty)
		"#,
	);
	query
		.push_values(
			courses,
			|mut query,
			 (id, map_id, stage, kzt, kzt_difficulty, skz, skz_difficutly, vnl, vnl_difficulty)| {
				query
					.push_bind(id)
					.push_bind(map_id)
					.push_bind(stage)
					.push_bind(kzt)
					.push_bind(kzt_difficulty)
					.push_bind(skz)
					.push_bind(skz_difficutly)
					.push_bind(vnl)
					.push_bind(vnl_difficulty);
			},
		)
		.build()
		.execute(&mut transaction)
		.await?;

	transaction.commit().await?;

	Ok(())
}

pub type RecordData = (u32, u32, u8, u32, u16, f64, u32, DateTime<Utc>);
pub async fn insert_records(records: &[RecordData], pool: &Pool<MySql>) -> Eyre<()> {
	let mut transaction = pool.begin().await?;

	let mut query = QueryBuilder::new(
		r#"
		INSERT INTO records
		  (id, course_id, mode_id, player_id, server_id, time, teleports, created_on)
		"#,
	);
	query
		.push_values(
			records,
			|mut query,
			 (id, course_id, mode_id, player_id, server_id, time, teleports, created_on)| {
				query
					.push_bind(id)
					.push_bind(course_id)
					.push_bind(mode_id)
					.push_bind(player_id)
					.push_bind(server_id)
					.push_bind(time)
					.push_bind(teleports)
					.push_bind(created_on);
			},
		)
		.build()
		.execute(&mut transaction)
		.await?;

	transaction.commit().await?;

	Ok(())
}
