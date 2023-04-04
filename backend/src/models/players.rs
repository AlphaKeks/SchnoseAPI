use {
	gokz_rs::SteamID,
	serde::{Deserialize, Serialize},
	sqlx::{types::Decimal, FromRow},
};

#[derive(Debug, Deserialize)]
pub struct PlayerParams {
	pub is_banned: Option<bool>,
	pub limit: Option<u32>,
	pub offset: Option<i32>,
}
/// `players` table
/// +-----------+------------------+------+-----+---------+-------+
/// | Field     | Type             | Null | Key | Default | Extra |
/// +-----------+------------------+------+-----+---------+-------+
/// | id        | int(10) unsigned | NO   | PRI | NULL    |       |
/// | name      | varchar(255)     | NO   | MUL | unknown |       |
/// | is_banned | tinyint(1)       | NO   |     | 0       |       |
/// +-----------+------------------+------+-----+---------+-------+
#[derive(Debug, FromRow)]
pub struct PlayerRow {
	pub id: u32,
	pub name: String,
	pub is_banned: bool,
	pub total_completions: i64,
	pub kzt_tp_completions: Decimal,
	pub kzt_pro_completions: Decimal,
	pub skz_tp_completions: Decimal,
	pub skz_pro_completions: Decimal,
	pub vnl_tp_completions: Decimal,
	pub vnl_pro_completions: Decimal,
}

#[derive(Debug, Serialize)]
pub struct PlayerResponse {
	pub name: String,
	pub steam_id: SteamID,
	pub is_banned: bool,
	pub records: RecordSummary,
}

#[derive(Debug, Serialize)]
pub struct RecordSummary {
	pub total: u32,
	pub kzt: RecordCount,
	pub skz: RecordCount,
	pub vnl: RecordCount,
}

#[derive(Debug, Serialize)]
pub struct RecordCount {
	pub tp: u32,
	pub pro: u32,
}

impl From<PlayerRow> for PlayerResponse {
	fn from(value: PlayerRow) -> Self {
		Self {
			name: value.name,
			steam_id: SteamID::from_id32(value.id),
			is_banned: value.is_banned,
			records: RecordSummary {
				total: value.total_completions as u32,
				kzt: RecordCount {
					tp: value
						.kzt_tp_completions
						.try_into()
						.unwrap(),
					pro: value
						.kzt_pro_completions
						.try_into()
						.unwrap(),
				},
				skz: RecordCount {
					tp: value
						.skz_tp_completions
						.try_into()
						.unwrap(),
					pro: value
						.skz_pro_completions
						.try_into()
						.unwrap(),
				},
				vnl: RecordCount {
					tp: value
						.vnl_tp_completions
						.try_into()
						.unwrap(),
					pro: value
						.vnl_pro_completions
						.try_into()
						.unwrap(),
				},
			},
		}
	}
}
