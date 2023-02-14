use {
	crate::migrations::sanitize,
	chrono::{DateTime, TimeZone, Utc},
	color_eyre::Result as Eyre,
	gokz_rs::{modes::APIMode, prelude::Mode},
	log::info,
	sqlx::{FromRow, MySql, Pool},
};

#[derive(Debug, Clone, FromRow)]
pub struct ModeSchema {
	pub id: u8,
	pub name: String,
	pub created_on: DateTime<Utc>,
}

impl TryFrom<APIMode> for ModeSchema {
	type Error = String;

	fn try_from(value: APIMode) -> Result<Self, Self::Error> {
		let Ok(mode) = Mode::try_from(value.id as u8) else {
    		return Err(String::from("bad modeid"));
    	};

		let Ok(created_on) = Utc.datetime_from_str(&value.updated_on, "%Y-%m-%dT%H:%M:%S") else {
			return Err(String::from("bad date"));
    	};

		Ok(Self {
			id: mode as u8,
			name: mode.api(),
			created_on,
		})
	}
}

pub const fn up() -> &'static str {
	r#"
CREATE TABLE
  IF NOT EXISTS modes (
    id TINYINT UNSIGNED NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
  );
"#
}

pub const fn down() -> &'static str {
	r#"DROP TABLE modes"#
}

pub async fn insert(data: &[ModeSchema], pool: &Pool<MySql>) -> Eyre<usize> {
	let mut transaction = pool.begin().await?;

	for (
		i,
		ModeSchema {
			id,
			name,
			created_on,
		},
	) in data.iter().enumerate()
	{
		let created_on = created_on.to_string();
		sqlx::query(&format!(
			r#"
			INSERT INTO modes
			  (id, name, created_on)
			VALUES
			  ({}, "{}", "{}")
			"#,
			id,
			sanitize(name),
			created_on.rsplit_once(' ').unwrap().0
		))
		.execute(&mut transaction)
		.await?;

		info!("{} / {}", i + 1, data.len());
	}

	transaction.commit().await?;

	Ok(data.len())
}
