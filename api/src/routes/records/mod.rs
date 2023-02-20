use {
	super::maps::Course,
	crate::util::number_to_bool,
	database::schemas::FancyPlayer,
	serde::{Deserialize, Serialize},
	sqlx::{types::time::PrimitiveDateTime, FromRow},
};

mod id;
pub(crate) use id::get as id;

mod index;
pub(crate) use index::get as index;

mod maptop;
pub(crate) use maptop::get as maptop;

// mod world_records;
// pub(crate) use world_records::get as world_records;

mod player;
pub(crate) use player::get as player;

#[derive(Debug, Clone, FromRow)]
pub struct RecordQuery {
	pub id: u32,
	pub map_name: String,
	pub course: String,
	pub mode: String,
	pub player: String,
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

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PlayerRowJSON {
	pub id: u32,
	pub name: String,
	#[serde(deserialize_with = "number_to_bool")]
	pub is_banned: bool,
}
