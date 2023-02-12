use {
	color_eyre::Result as Eyre,
	gokz_rs::{kzgo::maps::Response as KZGOMap, maps::Map},
	log::debug,
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
    id INT UNSIGNED NOT NULL PRIMARY KEY,
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

	debug!("{} global maps, {} kzgo maps", global_maps.len(), kzgo_maps.len());
	let maps = global_maps
		.into_iter()
		.map(|map| {
			debug!("currently @ {}", &map.name);
			let (kzgo_bonuses, kzgo_sp, kzgo_vp) = kzgo_maps
				.iter()
				.find(|kzgo_map| Some(&map.name) == kzgo_map.name.as_ref())
				.map(|kzgo_map| {
					debug!("FOUND {:?} ({})", &kzgo_map.name, &map.name);
					(
						kzgo_map.bonuses.unwrap_or(0),
						kzgo_map.sp.unwrap_or_default(),
						kzgo_map.vp.unwrap_or_default(),
					)
				})
				.unwrap_or_else(|| {
					debug!("DIDN'T FIND {}", &map.name);
					(0, false, false)
				});
			(map.id, map.name, map.difficulty, kzgo_bonuses, kzgo_sp, kzgo_vp)
		})
		.collect::<Vec<_>>();

	for (id, name, difficulty, kzgo_bonuses, kzgo_sp, kzgo_vp) in maps {
		for stage in 0..=kzgo_bonuses {
			let course_id = id * 100 + stage as i32;
			debug!("map {} with stage {} => course_id {}", id, stage, course_id);

			sqlx::query(&format!(
				r#"
				INSERT INTO courses
				  (id, map_id, stage, kzt, kzt_difficulty, skz, skz_difficulty, vnl, vnl_difficulty)
				VALUES
				  ({}, {}, {}, {}, {}, {}, {}, {}, {})
				"#,
				course_id,
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
			count += 1;
		}
	}

	transaction.commit().await?;

	Ok(count)
}
