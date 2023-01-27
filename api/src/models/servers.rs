use {
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
	pub approved_by: u64,
	pub approved_on: PrimitiveDateTime,
	pub updated_on: PrimitiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResponse {
	pub id: u16,
	pub name: String,
	pub owner_id: u64,
	pub approved_by: u64,
	pub approved_on: String,
	pub updated_on: String,
}

impl From<ServerModel> for ServerResponse {
	fn from(value: ServerModel) -> Self {
		Self {
			id: value.id,
			name: value.name,
			owner_id: value.owner_id,
			approved_by: value.approved_by,
			approved_on: value.approved_on.to_string(),
			updated_on: value.updated_on.to_string(),
		}
	}
}
