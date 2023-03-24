use {
	crate::{format_date, schemas::*},
	color_eyre::Result,
	log::info,
	sqlx::{MySql, Pool, QueryBuilder},
};

#[rustfmt::skip]
pub async fn modes(rows: &[ModeRow], pool: &Pool<MySql>) -> Result<()> {
	let mut transaction = pool.begin().await?;

	let mut query = QueryBuilder::new("INSERT INTO modes (id, name, created_on) ");

	for chunk in rows.chunks(64) {
		query.push_values(chunk, |mut query, ModeRow { id, name, created_on }| {
			query
				.push_bind(id)
				.push_bind(name)
				.push_bind(format_date(created_on));
		});

		info!("Inserting {} rows into `modes`.", chunk.len());

		query.build().execute(&mut transaction).await?;
		query.reset();
	}

	Ok(())
}

#[rustfmt::skip]
pub async fn players(rows: &[PlayerRow], pool: &Pool<MySql>) -> Result<()> {
	let mut transaction = pool.begin().await?;

	let mut query = QueryBuilder::new("INSERT INTO players (id, name, is_banned) ");

	for chunk in rows.chunks(64) {
		query.push_values(chunk, |mut query, PlayerRow { id, name, is_banned }| {
			query
				.push_bind(id)
				.push_bind(name)
				.push_bind(is_banned);
		});

		info!("Inserting {} rows into `players`.", chunk.len());

		query.build().execute(&mut transaction).await?;
		query.reset();
	}

	Ok(())
}

#[rustfmt::skip]
pub async fn courses(rows: &[CourseRow], pool: &Pool<MySql>) -> Result<()> {
	let mut transaction = pool.begin().await?;

	let mut query = QueryBuilder::new(
		"INSERT INTO courses (id, map_id, stage, kzt, kzt_difficulty, skz, skz_difficulty, vnl, vnl_difficulty) "
	);

	for chunk in rows.chunks(64) {
		query.push_values(chunk, |mut query, CourseRow { id, map_id, stage, kzt, kzt_difficulty, skz, skz_difficulty, vnl, vnl_difficulty }| {
			query
				.push_bind(id)
				.push_bind(map_id)
				.push_bind(stage)
				.push_bind(kzt)
				.push_bind(kzt_difficulty)
				.push_bind(skz)
				.push_bind(skz_difficulty)
				.push_bind(vnl)
				.push_bind(vnl_difficulty);
		});

		info!("Inserting {} rows into `courses`.", chunk.len());

		query.build().execute(&mut transaction).await?;
		query.reset();
	}

	Ok(())
}

#[rustfmt::skip]
pub async fn maps(rows: &[MapRow], pool: &Pool<MySql>) -> Result<()> {
	let mut transaction = pool.begin().await?;

	let mut query = QueryBuilder::new(
		"INSERT INTO maps (id, name, courses, validated, filesize, created_by, approved_by, created_on, updated_on) "
	);

	for chunk in rows.chunks(64) {
		query.push_values(chunk, |mut query, MapRow { id, name, courses, validated, filesize, created_by, approved_by, created_on, updated_on }| {
			query
				.push_bind(id)
				.push_bind(name)
				.push_bind(courses)
				.push_bind(validated)
				.push_bind(filesize)
				.push_bind(created_by)
				.push_bind(approved_by)
				.push_bind(format_date(created_on))
				.push_bind(format_date(updated_on));
		});

		info!("Inserting {} rows into `maps`.", chunk.len());

		query.build().execute(&mut transaction).await?;
		query.reset();
	}

	Ok(())
}

#[rustfmt::skip]
pub async fn servers(rows: &[ServerRow], pool: &Pool<MySql>) -> Result<()> {
	let mut transaction = pool.begin().await?;

	let mut query = QueryBuilder::new(
		"INSERT INTO servers (id, name, owned_by, approved_by) "
	);

	for chunk in rows.chunks(64) {
		query.push_values(chunk, |mut query, ServerRow { id, name, owned_by, approved_by }| {
			query
				.push_bind(id)
				.push_bind(name)
				.push_bind(owned_by)
				.push_bind(approved_by);
		});

		info!("Inserting {} rows into `servers`.", chunk.len());

		query.build().execute(&mut transaction).await?;
		query.reset();
	}

	Ok(())
}

#[rustfmt::skip]
pub async fn records(rows: &[RecordRow], pool: &Pool<MySql>) -> Result<()> {
	let mut transaction = pool.begin().await?;

	let mut query = QueryBuilder::new(
		"INSERT INTO records (id, course_id, mode_id, player_id, server_id, time, teleports, created_on) "
	);

	for chunk in rows.chunks(64) {
		query.push_values(chunk, |mut query, RecordRow { id, course_id, mode_id, player_id, server_id, time, teleports, created_on }| {
			query
				.push_bind(id)
				.push_bind(course_id)
				.push_bind(mode_id)
				.push_bind(player_id)
				.push_bind(server_id)
				.push_bind(time)
				.push_bind(teleports)
				.push_bind(format_date(created_on));
		});

		info!("Inserting {} rows into `records`.", chunk.len());

		query.build().execute(&mut transaction).await?;
		query.reset();
	}

	Ok(())
}
