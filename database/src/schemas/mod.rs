use {
	color_eyre::{eyre::eyre, Result as Eyre},
	gokz_rs::prelude::{Mode as GOKZMode, *},
	serde::{Deserialize, Serialize},
	sqlx::FromRow,
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
	let mut numbers = parts.split(':');
	let account_type = numbers.next()?.parse::<u32>().ok()?;
	let account_number = numbers.next()?.parse::<u32>().ok()?;
	Some(account_number * 2 + account_type)
}

pub mod raw;

impl From<&raw::ModeRow> for GOKZMode {
	fn from(value: &raw::ModeRow) -> Self {
		match value.id {
			200 => Self::KZTimer,
			201 => Self::SimpleKZ,
			202 => Self::Vanilla,
			_ => unimplemented!("Update `gokz_rs`"),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mode {
	pub id: u8,
	pub name: String,
	pub name_short: String,
	pub name_long: String,
	pub created_on: String,
}

impl From<raw::ModeRow> for Mode {
	fn from(value: raw::ModeRow) -> Self {
		let gokz_mode = GOKZMode::from(&value);
		Self {
			id: gokz_mode as u8,
			name: gokz_mode.api(),
			name_short: gokz_mode.short(),
			name_long: gokz_mode.to_string(),
			created_on: value.created_on,
		}
	}
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Player {
	pub account_id: u32,
	pub name: String,
	pub is_banned: bool,
	pub total_records: u32,
	pub kzt_tp_records: u32,
	pub kzt_pro_records: u32,
	pub skz_tp_records: u32,
	pub skz_pro_records: u32,
	pub vnl_tp_records: u32,
	pub vnl_pro_records: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FancyPlayer {
	pub account_id: u32,
	pub steam_id: SteamID,
	pub steam_id64: u64,
	pub name: String,
	pub is_banned: bool,
	pub total_records: u32,
	pub kzt_tp_records: u32,
	pub kzt_pro_records: u32,
	pub skz_tp_records: u32,
	pub skz_pro_records: u32,
	pub vnl_tp_records: u32,
	pub vnl_pro_records: u32,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Server {
	pub id: u16,
	pub name: String,
	pub owner_id: u32,
	pub owner_name: String,
	pub owner_is_banned: bool,
	pub approved_by_id: u32,
	pub approved_by_name: String,
	pub approved_by_is_banned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FancyServer {
	pub id: u16,
	pub name: String,
	pub owned_by: raw::PlayerRow,
	pub approved_by: raw::PlayerRow,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Map {
	pub id: u16,
	pub name: String,
	pub tier: u8,
	pub courses: u8,
	pub validated: bool,
	pub filesize: u64,
	pub creator_id: u32,
	pub creator_name: String,
	pub creator_is_banned: bool,
	pub approver_id: u32,
	pub approver_name: String,
	pub approver_is_banned: bool,
	pub created_on: String,
	pub updated_on: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FancyMap {
	pub id: u16,
	pub name: String,
	pub tier: Tier,
	pub courses: u8,
	pub validated: bool,
	pub filesize: u64,
	pub created_by: raw::PlayerRow,
	pub approved_by: raw::PlayerRow,
	pub created_on: String,
	pub updated_on: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
	pub id: u32,
	pub map: Map,
	pub stage: u8,
	pub kzt: bool,
	pub kzt_difficulty: Tier,
	pub skz: bool,
	pub skz_difficulty: Tier,
	pub vnl: bool,
	pub vnl_difficulty: Tier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
	pub id: u32,
	pub map: Map,
	pub course: Course,
	pub mode: Mode,
	pub player: raw::PlayerRow,
	pub server: Server,
	pub time: f64,
	pub teleports: u32,
	pub created_on: String,
}
