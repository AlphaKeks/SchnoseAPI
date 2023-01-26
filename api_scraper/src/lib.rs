use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedMap {
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
