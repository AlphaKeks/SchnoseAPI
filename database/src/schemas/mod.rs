use {
	color_eyre::{eyre::eyre, Result as Eyre},
	gokz_rs::prelude::{Mode as GOKZMode, *},
	serde::{Deserialize, Serialize},
	sqlx::{
		types::{time::PrimitiveDateTime, Decimal},
		FromRow,
	},
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
			created_on: value.created_on.to_string(),
		}
	}
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Player {
	pub account_id: u32,
	pub name: String,
	pub is_banned: bool,
	pub total_records: i64,
	pub kzt_tp_records: Decimal,
	pub kzt_pro_records: Decimal,
	pub skz_tp_records: Decimal,
	pub skz_pro_records: Decimal,
	pub vnl_tp_records: Decimal,
	pub vnl_pro_records: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FancyPlayer {
	pub name: String,
	pub steam_id: SteamID,
	pub steam_id64: String,
	pub account_id: u32,
	pub is_banned: bool,
	pub total_records: u32,
	pub kzt_tp_records: u32,
	pub kzt_pro_records: u32,
	pub skz_tp_records: u32,
	pub skz_pro_records: u32,
	pub vnl_tp_records: u32,
	pub vnl_pro_records: u32,
}

impl TryFrom<Player> for FancyPlayer {
	type Error = color_eyre::Report;

	fn try_from(value: Player) -> Result<Self, Self::Error> {
		let steam_id64 = account_id_to_steam_id64(value.account_id);
		let steam_id = SteamID::from(steam_id64);
		Ok(FancyPlayer {
			name: value.name,
			steam_id,
			steam_id64: steam_id64.to_string(),
			account_id: value.account_id,
			is_banned: value.is_banned,
			total_records: value.total_records.try_into()?,
			kzt_tp_records: value.kzt_tp_records.try_into()?,
			kzt_pro_records: value.kzt_pro_records.try_into()?,
			skz_tp_records: value.skz_tp_records.try_into()?,
			skz_pro_records: value.skz_pro_records.try_into()?,
			vnl_tp_records: value.vnl_tp_records.try_into()?,
			vnl_pro_records: value.vnl_pro_records.try_into()?,
		})
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactPlayer {
	pub name: String,
	pub steam_id64: String,
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

impl From<Server> for FancyServer {
	fn from(value: Server) -> Self {
		FancyServer {
			id: value.id,
			name: value.name,
			owned_by: raw::PlayerRow {
				id: value.owner_id,
				name: value.owner_name,
				is_banned: value.owner_is_banned,
			},
			approved_by: raw::PlayerRow {
				id: value.approved_by_id,
				name: value.approved_by_name,
				is_banned: value.approved_by_is_banned,
			},
		}
	}
}

#[derive(Debug, Clone, FromRow)]
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
	pub created_on: PrimitiveDateTime,
	pub updated_on: PrimitiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FancyMap {
	pub id: u16,
	pub name: String,
	pub tier: u8,
	pub courses: u8,
	pub validated: bool,
	pub filesize: String,
	pub created_by: raw::PlayerRow,
	pub approved_by: raw::PlayerRow,
	pub created_on: String,
	pub updated_on: String,
}

impl TryFrom<Map> for FancyMap {
	type Error = gokz_rs::prelude::Error;

	fn try_from(value: Map) -> Result<Self, Self::Error> {
		Ok(FancyMap {
			id: value.id,
			name: value.name,
			tier: Tier::try_from(value.tier)? as u8,
			courses: value.courses,
			validated: value.validated,
			filesize: value.filesize.to_string(),
			created_by: raw::PlayerRow {
				id: value.creator_id,
				name: value.creator_name,
				is_banned: value.creator_is_banned,
			},
			approved_by: raw::PlayerRow {
				id: value.approver_id,
				name: value.approver_name,
				is_banned: value.approver_is_banned,
			},
			created_on: value.created_on.to_string(),
			updated_on: value.updated_on.to_string(),
		})
	}
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Course {
	pub id: u32,
	pub map: FancyMap,
	pub stage: u8,
	pub kzt: bool,
	pub kzt_difficulty: u8,
	pub skz: bool,
	pub skz_difficulty: u8,
	pub vnl: bool,
	pub vnl_difficulty: u8,
}

#[derive(Debug, Clone, FromRow)]
pub struct Record {
	pub id: u32,

	pub map_id: u16,
	pub map_name: String,
	pub map_courses: u8,
	pub map_validated: bool,
	pub map_filesize: u64,
	pub map_created_by_id: u32,
	pub map_created_by_name: String,
	pub map_created_by_is_banned: bool,
	pub map_approved_by_id: u32,
	pub map_approved_by_name: String,
	pub map_approved_by_is_banned: bool,
	pub map_created_on: PrimitiveDateTime,
	pub map_updated_on: PrimitiveDateTime,

	pub course_id: u32,
	pub course_stage: u8,
	pub course_kzt: bool,
	pub course_kzt_difficulty: u8,
	pub course_skz: bool,
	pub course_skz_difficulty: u8,
	pub course_vnl: bool,
	pub course_vnl_difficulty: u8,

	pub mode_name: String,

	pub player_name: String,
	pub player_id: u32,

	pub server_name: String,

	pub time: f64,
	pub teleports: u32,
	pub created_on: PrimitiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FancyRecord {
	pub id: u32,
	pub map: FancyMap,
	pub course: Course,
	pub mode: String,
	pub player: CompactPlayer,
	pub server: String,
	pub time: f64,
	pub teleports: u32,
	pub created_on: String,
}

impl From<Record> for FancyRecord {
	fn from(value: Record) -> Self {
		let map = FancyMap {
			id: value.map_id,
			name: value.map_name,
			tier: value.course_kzt_difficulty,
			courses: value.map_courses,
			validated: value.map_validated,
			filesize: value.map_filesize.to_string(),
			created_by: raw::PlayerRow {
				id: value.map_created_by_id,
				name: value.map_created_by_name,
				is_banned: value.map_created_by_is_banned,
			},
			approved_by: raw::PlayerRow {
				id: value.map_approved_by_id,
				name: value.map_approved_by_name,
				is_banned: value.map_approved_by_is_banned,
			},
			created_on: value.map_created_on.to_string(),
			updated_on: value.map_updated_on.to_string(),
		};
		FancyRecord {
			id: value.id,
			map: map.clone(),
			course: Course {
				id: value.course_id,
				map,
				stage: value.course_stage,
				kzt: value.course_kzt,
				kzt_difficulty: value.course_kzt_difficulty,
				skz: value.course_skz,
				skz_difficulty: value.course_skz_difficulty,
				vnl: value.course_vnl,
				vnl_difficulty: value.course_vnl_difficulty,
			},
			mode: value.mode_name,
			player: CompactPlayer {
				name: value.player_name,
				steam_id64: (value.player_id as u64 + MAGIC_STEAM_ID_OFFSET).to_string(),
			},
			server: value.server_name,
			time: value.time,
			teleports: value.teleports,
			created_on: value.created_on.to_string(),
		}
	}
}
