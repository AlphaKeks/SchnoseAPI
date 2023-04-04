use {
	gokz_rs::{PlayerIdentifier, SteamID},
	serde::{Deserialize, Serialize},
	sqlx::FromRow,
};

#[derive(Debug, Deserialize)]
pub struct ServerParams {
	pub name: Option<String>,
	pub owned_by: Option<PlayerIdentifier>,
	pub approved_by: Option<PlayerIdentifier>,
	pub limit: Option<u32>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ServerRow {
	pub id: u16,
	pub name: String,
	pub owner_id: u32,
	pub owner_name: String,
	pub approver_id: u32,
	pub approver_name: String,
}

#[derive(Debug, Serialize)]
pub struct ServerResponse {
	pub id: u16,
	pub name: String,
	pub owner_name: String,
	pub owner_steam_id: SteamID,
	pub approver_name: String,
	pub approver_steam_id: SteamID,
}

impl From<ServerRow> for ServerResponse {
	fn from(value: ServerRow) -> Self {
		Self {
			id: value.id,
			name: value.name,
			owner_name: value.owner_name,
			owner_steam_id: SteamID::from_id32(value.owner_id),
			approver_name: value.approver_name,
			approver_steam_id: SteamID::from_id32(value.approver_id),
		}
	}
}
