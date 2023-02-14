use {
	gokz_rs::prelude::Mode as GOKZMode,
	serde::{Deserialize, Serialize},
	sqlx::FromRow,
};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ModeRow {
	pub id: u8,
	pub name: String,
	pub created_on: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PlayerRow {
	pub id: u32,
	pub name: String,
	pub is_banned: bool,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ServerRow {
	pub id: u16,
	pub name: String,
	pub owned_by: u32,
	pub approved_by: u32,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MapRow {
	pub id: u16,
	pub name: String,
	pub courses: u8,
	pub validated: bool,
	pub filesize: u64,
	pub created_by: u32,
	pub approved_by: u32,
	pub created_on: String,
	pub updated_on: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CourseRow {
	pub id: u32,
	pub map_id: u16,
	pub stage: u8,
	pub kzt: bool,
	pub kzt_difficulty: u8,
	pub skz: bool,
	pub skz_difficulty: u8,
	pub vnl: bool,
	pub vnl_difficulty: u8,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RecordRow {
	pub id: u32,
	pub course_id: u32,
	pub mode_id: u8,
	pub player_id: u32,
	pub server_id: u16,
	pub time: f64,
	pub teleports: u32,
	pub created_on: String,
}
