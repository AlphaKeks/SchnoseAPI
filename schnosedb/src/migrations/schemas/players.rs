use {
	crate::{migrations::sanitize, MAGIC_NUMBER},
	color_eyre::Result as Eyre,
	gokz_rs::players::Player,
	log::info,
	sqlx::{FromRow, MySql, Pool},
};

#[derive(Debug, Clone, FromRow)]
pub struct PlayerSchema {
	pub id: u32,
	pub name: String,
	pub is_banned: bool,
}

impl TryFrom<Player> for PlayerSchema {
	type Error = String;

	fn try_from(value: Player) -> Result<Self, Self::Error> {
		let Ok(steamid64) = value.steamid64.parse::<u64>() else {
			return Err(String::from("bad steamid64"))
		};

		let account_id = steamid64 - MAGIC_NUMBER;
		Ok(Self {
			id: account_id as u32,
			name: value.name,
			is_banned: value.is_banned,
		})
	}
}

pub const fn up() -> &'static str {
	r#"
CREATE TABLE
  IF NOT EXISTS players (
    id INT UNSIGNED NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL DEFAULT "unknown",
    is_banned BOOL NOT NULL DEFAULT FALSE
  );
"#
}

pub const fn down() -> &'static str {
	r#"DROP TABLE players"#
}

pub async fn insert(data: &[PlayerSchema], pool: &Pool<MySql>) -> Eyre<usize> {
	let mut transaction = pool.begin().await?;

	for (i, PlayerSchema { id, name, is_banned }) in data.iter().enumerate() {
		sqlx::query(&format!(
			r#"
			INSERT IGNORE INTO players
			  (id, name, is_banned)
			VALUES
			  ({}, "{}", {})
			"#,
			id,
			sanitize(name),
			is_banned
		))
		.execute(&mut transaction)
		.await?;

		info!("{} / {}", i + 1, data.len());
	}

	transaction.commit().await?;

	Ok(data.len())
}
