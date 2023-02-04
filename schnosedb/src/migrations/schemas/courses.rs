use {
	color_eyre::Result as Eyre,
	gokz_rs::{kzgo::maps::Response as KZGOMap, maps::Map},
	log::info,
	sqlx::{FromRow, MySql, Pool},
};

#[derive(Debug, Clone, FromRow)]
pub struct CourseSchema {
	pub id: u16,
	pub map_id: u16,
	pub stage: u8,
	pub kzt: bool,
	pub kzt_difficulty: u8,
	pub skz: bool,
	pub skz_difficulty: u8,
	pub vnl: bool,
	pub vnl_difficulty: u8,
}

pub const fn up() -> &'static str {
	r#"
CREATE TABLE
  IF NOT EXISTS courses (
    id SMALLINT UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT,
    map_id SMALLINT UNSIGNED NOT NULL,
    stage TINYINT UNSIGNED NOT NULL,
    kzt BOOLEAN NOT NULL,
    kzt_difficulty TINYINT NOT NULL,
    skz BOOLEAN NOT NULL,
    skz_difficulty TINYINT NOT NULL,
    vnl BOOLEAN NOT NULL,
    vnl_difficulty TINYINT NOT NULL,
    FOREIGN KEY (map_id) REFERENCES maps (id)
  );
"#
}

pub const fn down() -> &'static str {
	r#"DROP TABLE courses"#
}

pub async fn insert(
	global_maps: Vec<Map>,
	kzgo_maps: Vec<KZGOMap>,
	pool: &Pool<MySql>,
) -> Eyre<usize> {
	let mut transaction = pool.begin().await?;

	let mut count = 1;

	for (
		i,
		(
			Map {
				id,
				name,
				filesize: _,
				validated: _,
				difficulty,
				created_on: _,
				updated_on: _,
				approved_by_steamid64: _,
				workshop_url: _,
				download_url: _,
			},
			KZGOMap {
				_id: _,
				name: _,
				id: _,
				tier: _,
				workshopId: _,
				bonuses: kzgo_bonuses,
				sp: kzgo_sp,
				vp: kzgo_vp,
				mapperNames: _,
				mapperIds: _,
				date: _,
			},
		),
	) in global_maps
		.into_iter()
		.zip(kzgo_maps)
		.enumerate()
	{
		for stage in 0..kzgo_bonuses.unwrap_or(1) {
			sqlx::query! {
					r#"
			INSERT INTO courses
			  (map_id, stage, kzt, kzt_difficulty, skz, skz_difficulty, vnl, vnl_difficulty)
			VALUES
			  (?, ?, ?, ?, ?, ?, ?, ?)
						"#,
						id, stage, name.starts_with("skz_") || name.starts_with("vnl_"), difficulty, kzgo_sp, difficulty, kzgo_vp, difficulty
				}
					.execute(&mut transaction)
				.await?;

			info!("{} ({stage})", i + 1);
		}
	}

	transaction.commit().await?;

	Ok(count)
}
