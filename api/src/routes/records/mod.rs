use {
	super::maps::Course,
	database::schemas::FancyPlayer,
	serde::{Deserialize, Serialize},
	sqlx::{types::time::PrimitiveDateTime, FromRow},
};

mod id;
pub(crate) use id::get as id;

mod index;
pub(crate) use index::get as index;

mod player;
pub(crate) use player::get as player;

#[derive(Debug, Clone, FromRow)]
pub struct RecordQuery {
	pub id: u32,
	pub map_id: u16,
	pub map_name: String,
	pub course_id: u32,
	pub stage: u8,
	pub kzt: bool,
	pub kzt_difficulty: u8,
	pub skz: bool,
	pub skz_difficulty: u8,
	pub vnl: bool,
	pub vnl_difficulty: u8,
	pub mode: String,
	pub player_id: u32,
	pub player_name: String,
	pub player_is_banned: bool,
	pub server_name: String,
	pub time: f64,
	pub teleports: u32,
	pub created_on: PrimitiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
	pub id: u32,
	pub map_name: String,
	pub course: Course,
	pub mode: String,
	pub player: FancyPlayer,
	pub server_name: String,
	pub time: f64,
	pub teleports: u32,
	pub created_on: String,
}
