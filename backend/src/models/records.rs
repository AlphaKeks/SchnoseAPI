use {
	crate::{Error, Result},
	chrono::{DateTime, Utc},
	database::serialize_date,
	gokz_rs::{Mode, SteamID},
	serde::Serialize,
	sqlx::FromRow,
};

// #[derive(Debug, Deserialize)]
// pub struct RecordParams {
// }

/// +------------+----------------------+------+-----+---------------------+-------+
/// | Field      | Type                 | Null | Key | Default             | Extra |
/// +------------+----------------------+------+-----+---------------------+-------+
/// | id         | int(10) unsigned     | NO   | PRI | NULL                |       |
/// | course_id  | int(10) unsigned     | NO   | MUL | NULL                |       |
/// | mode_id    | tinyint(3) unsigned  | NO   | MUL | NULL                |       |
/// | player_id  | int(10) unsigned     | NO   | MUL | NULL                |       |
/// | server_id  | smallint(5) unsigned | NO   | MUL | NULL                |       |
/// | time       | double               | NO   | MUL | NULL                |       |
/// | teleports  | int(10) unsigned     | NO   |     | NULL                |       |
/// | created_on | datetime             | NO   | MUL | current_timestamp() |       |
/// +------------+----------------------+------+-----+---------------------+-------+
#[derive(Debug, FromRow)]
pub struct RecordRow {
	pub id: u32,
	pub player_name: String,
	pub player_id: u32,
	pub map_id: u16,
	pub map_name: String,
	pub stage: u8,
	pub mode_id: u8,
	pub time: f64,
	pub teleports: u32,
	pub created_on: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RecordResponse {
	pub id: u32,
	pub player_name: String,
	pub steam_id: SteamID,
	pub map_id: u16,
	pub map_name: String,
	pub stage: u8,
	pub mode: Mode,
	pub time: f64,
	pub teleports: u32,
	#[serde(serialize_with = "serialize_date")]
	pub created_on: DateTime<Utc>,
}

impl TryFrom<RecordRow> for RecordResponse {
	type Error = Error;

	fn try_from(value: RecordRow) -> Result<Self> {
		Ok(Self {
			id: value.id,
			player_name: value.player_name,
			steam_id: SteamID::from_id32(value.player_id),
			map_id: value.map_id,
			map_name: value.map_name,
			stage: value.stage,
			mode: value.mode_id.try_into()?,
			time: value.time,
			teleports: value.teleports,
			created_on: value.created_on,
		})
	}
}
