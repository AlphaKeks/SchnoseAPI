use {
	serde::{Deserialize, Serialize},
	sqlx::types::time::PrimitiveDateTime,
};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ModeModel {
	pub id: u8,
	pub name: String,
	pub name_short: String,
	pub name_long: String,
	pub created_on: PrimitiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeResponse {
	pub id: u8,
	pub name: String,
	pub name_short: String,
	pub name_long: String,
	pub created_on: String,
}

impl From<ModeModel> for ModeResponse {
	fn from(value: ModeModel) -> Self {
		Self {
			id: value.id,
			name: value.name,
			name_short: value.name_short,
			name_long: value.name_long,
			created_on: value.created_on.to_string(),
		}
	}
}
