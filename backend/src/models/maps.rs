use {
	crate::Error,
	database::{deserialize_date_opt, serialize_date},
	gokz_rs::{PlayerIdentifier, SteamID, Tier},
	serde::{Deserialize, Serialize},
	sqlx::{
		types::chrono::{DateTime, Utc},
		FromRow,
	},
	std::num::NonZeroU8,
};

#[derive(Debug, Deserialize)]
pub struct MapParams {
	pub name: Option<String>,
	pub tier: Option<Tier>,
	pub stages: Option<NonZeroU8>,
	pub validated: Option<bool>,
	pub mapper: Option<PlayerIdentifier>,
	pub approver: Option<PlayerIdentifier>,
	#[serde(deserialize_with = "deserialize_date_opt")]
	pub created_after: Option<DateTime<Utc>>,
	#[serde(deserialize_with = "deserialize_date_opt")]
	pub created_before: Option<DateTime<Utc>>,
	pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct MapResponse {
	pub id: u16,
	pub name: String,
	pub tier: Tier,
	pub courses: Vec<CourseRow>,
	pub validated: bool,
	pub filesize: u64,
	pub mapper_name: String,
	pub mapper_steam_id: SteamID,
	pub approver_name: String,
	pub approver_steam_id: SteamID,
	#[serde(serialize_with = "serialize_date")]
	pub created_on: DateTime<Utc>,
	#[serde(serialize_with = "serialize_date")]
	pub updated_on: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct MapRow {
	pub id: u16,
	pub name: String,
	pub json_courses: String,
	pub validated: bool,
	pub filesize: u64,
	pub mapper_id: u32,
	pub mapper_name: String,
	pub approver_id: u32,
	pub approver_name: String,
	pub created_on: DateTime<Utc>,
	pub updated_on: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct CourseRow {
	pub id: u32,
	pub stage: u8,
	pub kzt: bool,
	pub kzt_difficulty: Tier,
	pub skz: bool,
	pub skz_difficulty: Tier,
	pub vnl: bool,
	pub vnl_difficulty: Tier,
}

impl TryFrom<MapRow> for MapResponse {
	type Error = Error;

	fn try_from(value: MapRow) -> Result<Self, Self::Error> {
		let courses: Vec<database::schemas::CourseRow> = serde_json::from_str(&value.json_courses)?;

		Ok(Self {
			id: value.id,
			name: value.name,
			tier: courses[0].kzt_difficulty.try_into()?,
			courses: courses
				.into_iter()
				.filter_map(|row| {
					Some(CourseRow {
						id: row.id,
						stage: row.stage,
						kzt: row.kzt,
						kzt_difficulty: row.kzt_difficulty.try_into().ok()?,
						skz: row.skz,
						skz_difficulty: row.skz_difficulty.try_into().ok()?,
						vnl: row.vnl,
						vnl_difficulty: row.vnl_difficulty.try_into().ok()?,
					})
				})
				.collect(),
			validated: value.validated,
			filesize: value.filesize,
			mapper_name: value.mapper_name,
			mapper_steam_id: SteamID::from_id32(value.mapper_id),
			approver_name: value.approver_name,
			approver_steam_id: SteamID::from_id32(value.approver_id),
			created_on: value.created_on,
			updated_on: value.updated_on,
		})
	}
}
