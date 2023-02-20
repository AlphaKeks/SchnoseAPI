use {
	crate::util::number_to_bool,
	serde::{Deserialize, Serialize},
	sqlx::{types::time::PrimitiveDateTime, FromRow},
};

mod index;
pub(crate) use index::get as index;

mod ident;
pub(crate) use ident::get as ident;

#[derive(Debug, Clone, FromRow)]
pub struct MapRow {
	pub id: u16,
	pub name: String,
	pub courses: String,
	pub validated: bool,
	pub filesize: u64,
	pub mapper_name: String,
	pub created_by: u32,
	pub approver_name: String,
	pub approved_by: u32,
	pub created_on: PrimitiveDateTime,
	pub updated_on: PrimitiveDateTime,
}

#[derive(Debug, Clone, Copy, FromRow, Serialize, Deserialize)]
pub struct Course {
	pub id: u32,
	pub stage: u8,
	#[serde(deserialize_with = "number_to_bool")]
	pub kzt: bool,
	pub kzt_difficulty: u8,
	#[serde(deserialize_with = "number_to_bool")]
	pub skz: bool,
	pub skz_difficulty: u8,
	#[serde(deserialize_with = "number_to_bool")]
	pub vnl: bool,
	pub vnl_difficulty: u8,
}

#[derive(Debug, Serialize)]
pub struct Map {
	pub id: u16,
	pub name: String,
	pub tier: u8,
	pub courses: Vec<Course>,
	pub validated: bool,
	pub mapper_name: String,
	pub mapper_steam_id64: String,
	pub approver_name: String,
	pub approver_steam_id64: String,
	pub filesize: String,
	pub created_on: String,
	pub updated_on: String,
}
