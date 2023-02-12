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
    id SMALLINT UNSIGNED NOT NULL PRIMARY KEY,
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

	let mut kzgo_maps = kzgo_maps.into_iter();
	let maps = global_maps
		.into_iter()
		.map(|map| {
			let (kzgo_bonuses, kzgo_sp, kzgo_vp) = kzgo_maps
				.find(
					|kzgo_map| {
						if let Some(name) = &kzgo_map.name {
							name == &map.name
						} else {
							false
						}
					},
				)
				.map(|kzgo_map| {
					(
						kzgo_map.bonuses.unwrap_or(0),
						kzgo_map.sp.unwrap_or_default(),
						kzgo_map.vp.unwrap_or_default(),
					)
				})
				.unwrap_or((0, true, true));
			(map.id, map.name, map.difficulty, kzgo_bonuses, kzgo_sp, kzgo_vp)
		})
		.collect::<Vec<_>>();

	for (i, (id, name, difficulty, kzgo_bonuses, kzgo_sp, kzgo_vp)) in maps.into_iter().enumerate()
	{
		for stage in 0..kzgo_bonuses {
			sqlx::query(&format!(
				r#"
				INSERT INTO courses
				  (id, map_id, stage, kzt, kzt_difficulty, skz, skz_difficulty, vnl, vnl_difficulty)
				VALUES
				  ({}, {}, {}, {}, {}, {}, {}, {}, {})
				"#,
				id * 10 + stage as i32,
				id,
				stage,
				!name.starts_with("skz_") && !name.starts_with("vnl_"),
				difficulty,
				kzgo_sp,
				difficulty,
				kzgo_vp,
				difficulty
			))
			.execute(&mut transaction)
			.await?;

			info!("{} ({stage})", i + 1);
			count += 1;
		}
	}

	transaction.commit().await?;

	Ok(count)
}
