use {
	crate::{
		migrations::{self, util},
		MAGIC_NUMBER,
	},
	chrono::{DateTime, TimeZone, Utc},
	color_eyre::Result as Eyre,
	gokz_rs::{prelude::Mode, records::Record},
	log::{error, info},
	sqlx::{FromRow, MySql, Pool},
	std::time::Duration,
};

#[derive(Debug, Clone, FromRow)]
pub struct RecordSchema {
	pub id: u32,
	pub course_id: u16,
	pub mode_id: u8,
	pub player_id: u32,
	pub server_id: u16,
	pub time: f64,
	pub teleports: u32,
	pub created_on: DateTime<Utc>,
	pub __stage: u8,
}

impl TryFrom<Record> for RecordSchema {
	type Error = String;

	fn try_from(value: Record) -> Result<Self, Self::Error> {
		let Ok(created_on) = Utc.datetime_from_str(&value.created_on, "%Y-%m-%dT%H:%M:%S") else {
			return Err(String::from("bad date"));
    	};

		let Ok(mode) = value.mode.parse::<Mode>() else {
			return Err(String::from("bad mode"));
    	};

		let Ok(player_id) = value.steamid64.parse::<u64>() else {
			return Err(String::from("bad steamid"));
    	};

		Ok(Self {
			id: value.id as u32,
			course_id: 0,
			mode_id: mode as u8,
			player_id: (player_id - MAGIC_NUMBER) as u32,
			server_id: value.server_id as u16,
			time: value.time,
			teleports: value.teleports as u32,
			created_on,
			__stage: value.stage as u8,
		})
	}
}

#[derive(FromRow)]
struct CourseID(u16);

pub const fn up() -> &'static str {
	r#"
CREATE TABLE
  IF NOT EXISTS records (
    id INT UNSIGNED NOT NULL PRIMARY KEY,
    course_id INT UNSIGNED NOT NULL,
    mode_id TINYINT UNSIGNED NOT NULL,
    player_id INT UNSIGNED NOT NULL,
    server_id SMALLINT UNSIGNED NOT NULL,
    time DOUBLE NOT NULL,
    teleports INT NOT NULL,
    created_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (server_id) REFERENCES servers (id),
    FOREIGN KEY (player_id) REFERENCES players (id),
    FOREIGN KEY (course_id) REFERENCES courses (id),
    FOREIGN KEY (mode_id) REFERENCES modes (id)
  );
"#
}

pub const fn down() -> &'static str {
	r#"DROP TABLE records"#
}

pub async fn insert(
	data: &[RecordSchema],
	pool: &Pool<MySql>,
	steam_key: &str,
	gokz_client: &gokz_rs::Client,
) -> Eyre<usize> {
	let mut transaction = pool.begin().await?;

	for (
		i,
		RecordSchema {
			id,
			course_id: _,
			mode_id,
			mut player_id,
			server_id,
			time,
			teleports,
			created_on,
			__stage,
		},
	) in data.iter().enumerate()
	{
		if let Err(why) = sqlx::query(&format!(
			r#"
			SELECT * FROM players
			WHERE id = {player_id}
			"#
		))
		.fetch_one(pool)
		.await
		{
			error!("player `{player_id}` not in db. {why:?}");
			let steam_id64 = player_id as u64 + MAGIC_NUMBER;
			if let Ok(player) = util::get_player(steam_id64, steam_key, gokz_client).await {
				let player = migrations::schemas::players::PlayerSchema::try_from(player).unwrap();
				migrations::schemas::players::insert(&[player], pool).await?;
				std::thread::sleep(Duration::from_millis(500));
			} else {
				player_id = 0;
			};
		}

		let created_on = created_on.to_string();
		let CourseID(course_id) =
			sqlx::query_as::<_, CourseID>(&format!("SELECT id FROM courses WHERE map_id = {id}"))
				.fetch_one(pool)
				.await?;

		sqlx::query(&format!(
			r#"
			INSERT INTO records
			  (id, course_id, mode_id, player_id, server_id, time, teleports, created_on)
			VALUES
			  ({}, {}, {}, {}, {}, {}, {}, "{}")
			"#,
			id,
			course_id + *__stage as u16,
			mode_id,
			player_id,
			server_id,
			time,
			teleports,
			created_on.rsplit_once(' ').unwrap().0
		))
		.execute(&mut transaction)
		.await?;

		info!("{} / {}", i + 1, data.len());
	}

	transaction.commit().await?;

	Ok(data.len())
}
