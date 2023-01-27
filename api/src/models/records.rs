use {
	serde::{Deserialize, Serialize},
	sqlx::types::time::PrimitiveDateTime,
};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecordsQuery {
	pub player_id: Option<u64>,
	pub mode_id: Option<u8>,
	pub has_teleports: Option<bool>,
	pub stage: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecentQuery {
	pub mode_id: Option<u8>,
	pub has_teleports: Option<bool>,
	pub limit: Option<u32>,
}

// TODO
// #[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
// pub struct PBQuery {
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
// pub struct WRQuery {
// }

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RecordModel {
	pub id: u32,
	pub map_id: u16,
	pub mode_id: u8,
	pub player_id: u64,
	pub server_id: u16,
	pub stage: u8,
	pub teleports: u16,
	pub time: f64,
	pub created_on: PrimitiveDateTime,
	pub global_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordResponse {
	pub id: u32,
	pub map_id: u16,
	pub mode_id: u8,
	pub player_id: u64,
	pub server_id: u16,
	pub stage: u8,
	pub teleports: u16,
	pub time: f64,
	pub created_on: String,
	pub global_id: u32,
}

impl From<RecordModel> for RecordResponse {
	fn from(value: RecordModel) -> Self {
		Self {
			id: value.id,
			map_id: value.map_id,
			mode_id: value.mode_id,
			player_id: value.player_id,
			server_id: value.server_id,
			stage: value.stage,
			teleports: value.teleports,
			time: value.time,
			created_on: value.created_on.to_string(),
			global_id: value.global_id,
		}
	}
}
