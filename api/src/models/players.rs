use {
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
	pub is_banned: bool,
	pub first_login: String,
	pub last_login: String,
	pub playtime: String,
}

impl From<PlayerModel> for PlayerResponse {
	fn from(value: PlayerModel) -> Self {
		Self {
			id: value.id,
			name: value.name,
			is_banned: value.is_banned,
			first_login: value.first_login.to_string(),
			last_login: value.last_login.to_string(),
			playtime: value.playtime.to_string(),
		}
	}
}
