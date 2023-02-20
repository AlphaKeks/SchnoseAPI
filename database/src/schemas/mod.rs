use {
	color_eyre::{eyre::eyre, Result as Eyre},
	gokz_rs::prelude::Mode as GOKZMode,
	serde::{Deserialize, Serialize},
	sqlx::{types::time::PrimitiveDateTime, FromRow},
};

pub const MAGIC_STEAM_ID_OFFSET: u64 = 76561197960265728;
pub const fn account_id_to_steam_id64(account_id: u32) -> u64 {
	account_id as u64 + MAGIC_STEAM_ID_OFFSET
}
pub fn steam_id64_to_account_id(steam_id64: u64) -> Eyre<u32> {
	if steam_id64 > MAGIC_STEAM_ID_OFFSET {
		Ok((steam_id64 - MAGIC_STEAM_ID_OFFSET) as u32)
	} else {
		Err(eyre!("BAD STEAMID64"))
	}
}
pub fn steam_id_to_account_id(steam_id: &str) -> Option<u32> {
	let (_, parts) = steam_id.split_once('_')?;
	let numbers = parts
		.split(':')
		.skip(1)
		.filter_map(|num| num.parse::<u32>().ok())
		.collect::<Vec<_>>();
	Some(numbers.get(1)? * 2 + numbers.first()?)
}

#[derive(Debug, Clone, FromRow)]
pub struct ModeRow {
	pub id: u8,
	pub name: String,
	pub created_on: PrimitiveDateTime,
}

impl From<&ModeRow> for GOKZMode {
	fn from(value: &ModeRow) -> Self {
		match value.id {
			200 => Self::KZTimer,
			201 => Self::SimpleKZ,
			202 => Self::Vanilla,
			_ => unimplemented!("Update `gokz_rs`"),
		}
	}
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PlayerRow {
	pub id: u32,
	pub name: String,
	pub is_banned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FancyPlayer {
	pub id: u32,
	pub name: String,
	pub steam_id: String,
	pub steam_id64: String,
	pub is_banned: bool,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ServerRow {
	pub id: u16,
	pub name: String,
	pub owned_by: u32,
	pub approved_by: u32,
}

#[derive(Debug, Clone, FromRow)]
pub struct MapRow {
	pub id: u16,
	pub name: String,
	pub courses: u8,
	pub validated: bool,
	pub filesize: u64,
	pub created_by: u32,
	pub approved_by: u32,
	pub created_on: PrimitiveDateTime,
	pub updated_on: PrimitiveDateTime,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CourseRow {
	pub id: u32,
	pub map_id: u16,
	pub stage: u8,
	pub kzt: bool,
	pub kzt_difficulty: u8,
	pub skz: bool,
	pub skz_difficulty: u8,
	pub vnl: bool,
	pub vnl_difficulty: u8,
}

#[derive(Debug, Clone, FromRow)]
pub struct RecordRow {
	pub id: u32,
	pub course_id: u32,
	pub mode_id: u8,
	pub player_id: u32,
	pub server_id: u16,
	pub time: f64,
	pub teleports: u32,
	pub created_on: PrimitiveDateTime,
}
