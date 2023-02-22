use {
	serde::{de::Error, Deserialize, Deserializer, Serialize},
	sqlx::{types::time::PrimitiveDateTime, FromRow},
};

mod index;
pub(crate) use index::get as index;

mod ident;
pub(crate) use ident::get as ident;

#[derive(Debug, Clone, FromRow)]
pub(crate) struct MapRow {
	pub(crate) id: u16,
	pub(crate) name: String,
	pub(crate) courses: String,
	pub(crate) validated: bool,
	pub(crate) filesize: u64,
	pub(crate) mapper_name: String,
	pub(crate) created_by: u32,
	pub(crate) approver_name: String,
	pub(crate) approved_by: u32,
	pub(crate) created_on: PrimitiveDateTime,
	pub(crate) updated_on: PrimitiveDateTime,
}

#[derive(Debug, Clone, Copy, FromRow, Serialize, Deserialize)]
pub struct Course {
	pub(crate) id: u32,
	pub(crate) stage: u8,
	#[serde(deserialize_with = "number_to_bool")]
	pub(crate) kzt: bool,
	pub(crate) kzt_difficulty: u8,
	#[serde(deserialize_with = "number_to_bool")]
	pub(crate) skz: bool,
	pub(crate) skz_difficulty: u8,
	#[serde(deserialize_with = "number_to_bool")]
	pub(crate) vnl: bool,
	pub(crate) vnl_difficulty: u8,
}

#[derive(Debug, Serialize)]
pub(crate) struct Map {
	pub(crate) id: u16,
	pub(crate) name: String,
	pub(crate) tier: u8,
	pub(crate) courses: Vec<Course>,
	pub(crate) validated: bool,
	pub(crate) mapper_name: String,
	pub(crate) mapper_steam_id64: String,
	pub(crate) approver_name: String,
	pub(crate) approver_steam_id64: String,
	pub(crate) filesize: String,
	pub(crate) created_on: String,
	pub(crate) updated_on: String,
}

pub fn number_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
	D: Deserializer<'de>,
{
	let num = i32::deserialize(deserializer)?;
	if num == 1 {
		Ok(true)
	} else if num == 0 {
		Ok(false)
	} else {
		Err(Error::custom(crate::Error::JSON))
	}
}
