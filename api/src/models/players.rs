use {
	gokz_rs::prelude::*,
	serde::{Deserialize, Serialize},
	sqlx::types::time::{PrimitiveDateTime, Time},
};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PlayerQuery {
	pub limit: Option<u64>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PlayerModel {
	pub id: u64,
	pub name: String,
	pub is_banned: bool,
	pub first_login: PrimitiveDateTime,
	pub last_login: PrimitiveDateTime,
	pub playtime: Time,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerResponse {
	pub id: u64,
	pub name: String,
	pub steam_id: String,
	pub is_banned: bool,
}

impl From<PlayerModel> for PlayerResponse {
	fn from(value: PlayerModel) -> Self {
		Self {
			id: value.id,
			name: value.name,
			steam_id: SteamID::from(value.id).to_string(),
			is_banned: value.is_banned,
		}
	}
}
