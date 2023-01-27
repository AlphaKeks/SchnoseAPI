use {
	serde::{Deserialize, Serialize},
	sqlx::types::time::PrimitiveDateTime,
};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MapQuery {
	pub name: Option<String>,
	pub validated: Option<bool>,
	pub created_by: Option<u64>,
	pub created_on: Option<String>,
	pub limit: Option<u64>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MapModel {
	pub id: u16,
	pub name: String,
	pub difficulty: u8,
	pub validated: bool,
	pub filesize: u64,
	pub created_by: u64,
	pub approved_by: u64,
	pub created_on: PrimitiveDateTime,
	pub updated_on: PrimitiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapResponse {
	pub id: u16,
	pub name: String,
	pub difficulty: u8,
	pub validated: bool,
	pub filesize: u64,
	pub created_by: u64,
	pub approved_by: u64,
	pub created_on: String,
	pub updated_on: String,
}

impl From<MapModel> for MapResponse {
	fn from(value: MapModel) -> Self {
		Self {
			id: value.id,
			name: value.name,
			difficulty: value.difficulty,
			validated: value.validated,
			filesize: value.filesize,
			created_by: value.created_by,
			approved_by: value.approved_by,
			created_on: value.created_on.to_string(),
			updated_on: value.updated_on.to_string(),
		}
	}
}
