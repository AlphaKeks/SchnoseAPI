use {
	crate::MAGIC_NUMBER,
	chrono::{DateTime, TimeZone, Utc},
	color_eyre::Result as Eyre,
	gokz_rs::{kzgo::maps::Response as KZGOMap, maps::Map},
	log::info,
	sqlx::{FromRow, MySql, Pool},
};

#[derive(Debug, Clone, FromRow)]
pub struct MapSchema {
	pub id: u16,
	pub name: String,
	pub courses: u8,
	pub validated: bool,
	pub filesize: u64,
	pub created_by: u32,
	pub approved_by: u32,
	pub created_on: DateTime<Utc>,
	pub updated_on: DateTime<Utc>,
}

impl TryFrom<Map> for MapSchema {
	type Error = String;

	fn try_from(value: Map) -> Result<Self, Self::Error> {
		let Ok(approved_by) = value.approved_by_steamid64.parse::<u64>() else {
			return Err(String::from("bad steamid"));
		};

		let Ok(created_on) = Utc.datetime_from_str(&value.created_on, "%Y-%m-%dT%H:%M:%S") else {
			return Err(String::from("bad date"));
        };

		let Ok(updated_on) = Utc.datetime_from_str(&value.updated_on, "%Y-%m-%dT%H:%M:%S") else {
			return Err(String::from("bad date"));
        };

		Ok(Self {
			id: value.id as u16,
			name: value.name,
			courses: 0,
			validated: value.validated,
			filesize: value.filesize as u64,
			created_by: 0,
			approved_by: (approved_by - MAGIC_NUMBER) as u32,
			created_on,
			updated_on,
		})
	}
}

pub const fn up() -> &'static str {
	r#"
CREATE TABLE
  IF NOT EXISTS maps (
    id SMALLINT UNSIGNED NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    courses TINYINT UNSIGNED NOT NULL DEFAULT 1,
    validated BOOLEAN NOT NULL DEFAULT FALSE,
    filesize BIGINT UNSIGNED NOT NULL,
    created_by INT UNSIGNED NOT NULL,
    approved_by INT UNSIGNED NOT NULL,
    created_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES players (id),
    FOREIGN KEY (approved_by) REFERENCES players (id)
  );
"#
}

pub const fn down() -> &'static str {
	r#"DROP TABLE maps"#
}

pub async fn insert(
	data: &[MapSchema],
	kzgo_maps: Vec<KZGOMap>,
	pool: &Pool<MySql>,
) -> Eyre<usize> {
	let mut transaction = pool.begin().await?;

	for (
		i,
		MapSchema {
			id,
			name,
			mut courses,
			validated,
			filesize,
			created_by,
			approved_by,
			created_on,
			updated_on,
		},
	) in data.iter().enumerate()
	{
		let created_on = created_on.to_string();
		let updated_on = updated_on.to_string();
		let kzgo_map = kzgo_maps
			.iter()
			.find(|map| map.name.as_ref().unwrap().eq(name))
			.map(|map| map.to_owned())
			.unwrap_or(KZGOMap {
				_id: None,
				name: None,
				id: None,
				tier: None,
				workshopId: None,
				bonuses: Some(0),
				sp: None,
				vp: None,
				mapperNames: vec![None],
				mapperIds: vec![None],
				date: None,
			});

		courses = kzgo_map.bonuses.unwrap();

		sqlx::query! {
			r#"
			INSERT INTO maps
			  (id, name, courses, validated, filesize, created_by, approved_by, created_on, updated_on)
			VALUES
			  (?, ?, ?, ?, ?, ?, ?, ?, ?)
			"#,
			id,
			name,
			courses,
			validated,
			filesize,
			created_by,
			approved_by,
			created_on.rsplit_once(' ').unwrap().0,
			updated_on.rsplit_once(' ').unwrap().0
		}
		.execute(&mut transaction)
		.await?;

		info!("{} / {}", i + 1, data.len());
	}

	transaction.commit().await?;

	Ok(data.len())
}
