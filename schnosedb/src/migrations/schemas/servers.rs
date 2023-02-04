use {
	crate::{migrations::sanitize, MAGIC_NUMBER},
	color_eyre::Result as Eyre,
	gokz_rs::servers::Server,
	log::info,
	sqlx::{FromRow, MySql, Pool},
};

#[derive(Debug, Clone, FromRow)]
pub struct ServerSchema {
	pub id: u16,
	pub name: String,
	pub owned_by: u32,
	pub approved_by: u32,
}

impl TryFrom<Server> for ServerSchema {
	type Error = String;

	fn try_from(value: Server) -> Result<Self, Self::Error> {
		let Ok(owned_by) = value.owner_steamid64.parse::<u64>() else {
			return Err(String::from("bad ownerid"));
        };

		Ok(Self {
			id: value.id as u16,
			name: value.name,
			owned_by: (owned_by - MAGIC_NUMBER) as u32,
			approved_by: 331763898, // gosh
		})
	}
}

pub const fn up() -> &'static str {
	r#"
CREATE TABLE
  IF NOT EXISTS servers (
    id SMALLINT UNSIGNED NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    owned_by INT UNSIGNED NOT NULL,
    approved_by INT UNSIGNED NOT NULL,
    FOREIGN KEY (owned_by) REFERENCES players (id),
    FOREIGN KEY (approved_by) REFERENCES players (id)
  );
"#
}

pub const fn down() -> &'static str {
	r#"DROP TABLE servers"#
}

pub async fn insert(data: &[ServerSchema], pool: &Pool<MySql>) -> Eyre<usize> {
	let mut transaction = pool.begin().await?;

	for (i, ServerSchema { id, name, owned_by, approved_by }) in data.iter().enumerate() {
		sqlx::query! {
			r#"
			INSERT INTO servers
			  (id, name, owned_by, approved_by)
			VALUES
			  (?, ?, ?, ?)
			"#,
			id,
			sanitize(name),
			owned_by,
			approved_by
		}
		.execute(&mut transaction)
		.await?;

		info!("{} / {}", i + 1, data.len());
	}

	transaction.commit().await?;

	Ok(data.len())
}
