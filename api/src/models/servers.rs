use {
	gokz_rs::prelude::*,
	serde::{Deserialize, Serialize},
	sqlx::types::time::PrimitiveDateTime,
};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServerQuery {
	pub name: Option<String>,
	pub owner_id: Option<u64>,
	pub limit: Option<u64>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ServerModel {
	pub id: u16,
	pub name: String,
	pub owner_id: u64,
	pub owner_name: String,
	pub approved_by_id: u64,
	pub approved_by_name: String,
	pub approved_on: PrimitiveDateTime,
	pub updated_on: PrimitiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResponse {
	pub id: u16,
	pub name: String,
	pub owner_id: u64,
	pub owner_name: String,
	pub owner_steam_id: String,
	pub approver_id: u64,
	pub approver_name: String,
	pub approver_steam_id: String,
	pub approved_on: String,
	pub updated_on: String,
}

impl From<ServerModel> for ServerResponse {
	fn from(value: ServerModel) -> Self {
		Self {
			id: value.id,
			name: value.name,
			owner_id: value.owner_id,
			owner_name: value.owner_name,
			owner_steam_id: SteamID::from(value.owner_id).to_string(),
			approver_id: value.approved_by_id,
			approver_name: value.approved_by_name,
			approver_steam_id: SteamID::from(value.approved_by_id).to_string(),
			approved_on: value.approved_on.to_string(),
			updated_on: value.updated_on.to_string(),
		}
	}
}
