use {
	chrono::{DateTime, Utc},
	color_eyre::Result as Eyre,
	log::{debug, info, trace},
	sqlx::{MySql, Pool},
};

pub async fn insert<RowData, Query>(
	table_name: &str,
	rows: &[RowData],
	query: Query,
	pool: &Pool<MySql>,
) -> Eyre<()>
where
	Query: Fn(&RowData) -> String,
{
	let total_rows = rows.len();
	trace!("Inserting {total_rows} rows into `{table_name}`...");

	let mut transaction = pool.begin().await?;
	for (i, row) in rows.iter().enumerate() {
		let query = query(row);
		debug!("[{} / {}] Query:\n{}", i + 1, total_rows, &query);

		sqlx::query(&query)
			.execute(&mut transaction)
			.await?;
		trace!("Inserted {} / {} rows.", i + 1, total_rows);
	}

	transaction.commit().await?;
	info!("Successfully inserted {total_rows} into `modes`.");

	Ok(())
}

pub type ModeData = (u8, String, DateTime<Utc>);
pub async fn insert_modes(modes: &[ModeData], pool: &Pool<MySql>) -> Eyre<()> {
	let query = |(id, name, created_on): &ModeData| {
		format!(
			r#"
			INSERT INTO modes
			  (id, name, created_on)
			VALUES
			  ({id}, "{name}", "{created_on}")
			"#
		)
	};

	insert("modes", modes, query, pool).await?;

	Ok(())
}

pub type PlayerData = (u32, String, bool);
pub async fn insert_players(players: &[PlayerData], pool: &Pool<MySql>) -> Eyre<()> {
	let query = |(id, name, is_banned): &PlayerData| {
		format!(
			r#"
			INSERT INTO players
			  (id, name, is_banned)
			VALUES
			  ({id}, "{name}", "{is_banned}")
			"#
		)
	};

	insert("players", players, query, pool).await?;

	Ok(())
}

pub type ServerData = (u16, String, u32, u32);
pub async fn insert_servers(servers: &[ServerData], pool: &Pool<MySql>) -> Eyre<()> {
	let query = |(id, name, owned_by, approved_by): &ServerData| {
		format!(
			r#"
			INSERT INTO servers
			  (id, name, owned_by, approved_by)
			VALUES
			  ({id}, "{name}", {owned_by}, {approved_by})
			"#
		)
	};

	insert("servers", servers, query, pool).await?;

	Ok(())
}

pub type MapData = (
	u16,
	String,
	u8,
	bool,
	u64,
	u32,
	u32,
	DateTime<Utc>,
	DateTime<Utc>,
);
pub async fn insert_maps(maps: &[MapData], pool: &Pool<MySql>) -> Eyre<()> {
	let query = |(
		id,
		name,
		courses,
		validated,
		filesize,
		created_by,
		approved_by,
		created_on,
		updated_on,
	): &MapData| {
		format!(
			r#"
			INSERT INTO maps
			  (id, name, courses, validated, filesize,
			   created_by, approved_by, created_on, updated_on)
			VALUES
			  ({id}, "{name}", {courses}, {validated}, {filesize},
			   {created_by}, {approved_by}, "{created_on}", "{updated_on}")
			"#
		)
	};

	insert("maps", maps, query, pool).await?;

	Ok(())
}

pub type CourseData = (u32, u16, u8, bool, u8, bool, u8, bool, u8);
pub async fn insert_courses(courses: &[CourseData], pool: &Pool<MySql>) -> Eyre<()> {
	let query = |(
		id,
		map_id,
		stage,
		kzt,
		kzt_difficulty,
		skz,
		skz_difficutly,
		vnl,
		vnl_difficulty,
	): &CourseData| {
		format!(
			r#"
			INSERT INTO courses
			  (id, map_id, stage, kzt, kzt_difficulty, skz, skz_difficutly, vnl, vnl_difficulty)
			VALUES
			  ({id}, {map_id}, {stage}, {kzt}, {kzt_difficulty}, {skz}, {skz_difficutly}, {vnl},
			   {vnl_difficulty})
			"#
		)
	};

	insert("courses", courses, query, pool).await?;

	Ok(())
}

pub type RecordData = (u32, u32, u8, u32, u16, f64, u32, DateTime<Utc>);
pub async fn insert_records(records: &[RecordData], pool: &Pool<MySql>) -> Eyre<()> {
	let query = |(
	id, course_id, mode_id, player_id, server_id, time, teleports, created_on
	): &RecordData| {
		let created_on = created_on.format("%Y-%m-%d %H:%M:%S");
		format!(
			r#"
			INSERT INTO records
			  (id, course_id, mode_id, player_id, server_id, time, teleports, created_on)
			VALUES
			  ({id}, {course_id}, {mode_id}, {player_id}, {server_id}, {time}, {teleports},
			   "{created_on}")
			"#
		)
	};

	insert("records", records, query, pool).await?;

	Ok(())
}
