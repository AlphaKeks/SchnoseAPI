use {
	crate::{
		migrations::{self, schemas::courses::CourseSchema, util},
		MAGIC_NUMBER,
	},
	chrono::{DateTime, TimeZone, Utc},
	color_eyre::Result as Eyre,
	gokz_rs::{prelude::Mode, records::Record},
	log::{debug, error, info},
	serde::{Deserialize, Serialize},
	sqlx::{FromRow, MySql, Pool},
	std::time::Duration,
};

#[derive(Debug, Clone, FromRow)]
pub struct RecordSchema {
	pub id: u32,
	pub course_id: u16,
	pub mode_id: u8,
	pub player_id: u32,
	pub time: f64,
	pub teleports: u32,
	pub created_on: DateTime<Utc>,
	pub __server_name: String,
	pub __stage: u8,
	pub __map_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElasticRecord {
	pub id: u32,
	pub steamid64: u64,
	pub player_name: String,
	pub steam_id: String,
	pub server_name: String,
	pub map_name: String,
	pub stage: u8,
	pub mode: String,
	pub tickrate: u8,
	pub time: f64,
	pub teleports: u32,
	pub created_on: String,
}

impl TryFrom<ElasticRecord> for RecordSchema {
	type Error = String;

	fn try_from(value: ElasticRecord) -> Result<Self, Self::Error> {
		let Ok(created_on) = Utc.datetime_from_str(&value.created_on, "%Y-%m-%dT%H:%M:%S") else {
			return Err(String::from("bad date"));
    	};

		let Ok(mode) = value.mode.parse::<Mode>() else {
			return Err(String::from("bad mode"));
    	};

		let player_id = u32::try_from(value.steamid64 - MAGIC_NUMBER)
			.map_err(|_| String::from("bad steamid64"))?;

		Ok(Self {
			id: value.id,
			course_id: 0,
			mode_id: mode as u8,
			player_id,
			time: value.time,
			teleports: value.teleports,
			created_on,
			__server_name: value.server_name,
			__stage: value.stage,
			__map_name: value.map_name,
		})
	}
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
			time: value.time,
			teleports: value.teleports as u32,
			created_on,
			__server_name: value
				.server_name
				.unwrap_or_else(|| String::from("unknown")),
			__stage: value.stage as u8,
			__map_name: value
				.map_name
				.unwrap_or_else(|| String::from("unknown")),
		})
	}
}

#[derive(FromRow)]
struct ServerID(u16);

#[derive(FromRow)]
struct CourseID(u32);

#[derive(FromRow)]
struct MapID(u16);

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
			time,
			teleports,
			created_on,
			__server_name,
			__stage,
			__map_name,
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
			} else {
				debug!("player {steam_id64} doesn't exist");
				player_id = 0;
			};
			std::thread::sleep(Duration::from_millis(500));
		}

		let created_on = created_on.to_string();
		let Ok(MapID(map_id)) = sqlx::query_as(
			&format!(r#"SELECT id FROM maps WHERE name = "{__map_name}""#)
		)
		.fetch_one(pool)
		.await else { continue; };

		let CourseID(course_id) = match sqlx::query_as::<_, CourseID>(&format!(
			r#"SELECT id FROM courses WHERE map_id = {}"#,
			map_id as u32 * 100 + *__stage as u32
		))
		.fetch_one(pool)
		.await
		{
			// cool
			Ok(course_id) => course_id,
			// Probably only main course in db because map was not global.
			// So we just take the main course and copy it.
			_ => {
				let CourseSchema {
					id,
					map_id,
					stage,
					kzt,
					kzt_difficulty,
					skz,
					skz_difficulty,
					vnl,
					vnl_difficulty,
				} = sqlx::query_as::<_, CourseSchema>(&format!(
					r#"SELECT * FROM courses WHERE map_id = {map_id}"#
				))
				.fetch_one(pool)
				.await?;

				let new_id = id + *__stage as u32;

				sqlx::query(&format!(
					r#"
					INSERT INTO courses
					  (id, map_id, stage, kzt, kzt_difficulty, skz, skz_difficulty, vnl, vnl_difficulty)
					VALUES
					  ({new_id}, {map_id}, {stage}, {kzt}, {kzt_difficulty}, {skz}, {skz_difficulty}, {vnl}, {vnl_difficulty})
					"#,
				))
				.execute(pool)
				.await?;

				CourseID(new_id)
			}
		};

		let ServerID(server_id) = sqlx::query_as::<_, ServerID>(&format!(
			r#"SELECT id FROM servers WHERE name = "{__server_name}""#
		))
		.fetch_one(pool)
		.await
		.unwrap_or(ServerID(0));

		sqlx::query(&format!(
			r#"
			INSERT INTO records
			  (id, course_id, mode_id, player_id, server_id, time, teleports, created_on)
			VALUES
			  ({}, {}, {}, {}, {}, {}, {}, "{}")
			"#,
			id,
			course_id,
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
