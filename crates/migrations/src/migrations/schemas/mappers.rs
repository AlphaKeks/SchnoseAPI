use {
	crate::MAGIC_NUMBER,
	color_eyre::Result as Eyre,
	log::info,
	serde::{Deserialize, Serialize},
	sqlx::{MySql, Pool},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KZGOInput {
	pub name: String,
	pub mapper_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
	pub map_name: String,
	pub mapper_name: String,
	pub mapper_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroInput {
	pub id: Option<String>,
	pub name: String,
	pub difficulty: Option<String>,
	pub workshop_url: Option<String>,
	pub mapper_name: Option<String>,
	pub mapper_steamid64: String,
}

#[allow(clippy::upper_case_acronyms)]
pub enum InputKind {
	KZGO(Vec<KZGOInput>),
	Zero(Vec<ZeroInput>),
}

pub async fn update(data: InputKind, pool: &Pool<MySql>) -> Eyre<usize> {
	let mut transaction = pool.begin().await?;

	match data {
		InputKind::KZGO(data) => {
			for (i, KZGOInput { name, mapper_id }) in data.iter().enumerate() {
				let Ok(mapper_id) = mapper_id.parse::<u64>() else {
					continue;
				};

				if mapper_id <= MAGIC_NUMBER {
					continue;
				}

				let mapper_id = mapper_id - MAGIC_NUMBER;

				sqlx::query(&format!(
					r#"
			UPDATE maps
			SET created_by = {mapper_id}
			WHERE name = "{name}"
			"#
				))
				.execute(&mut transaction)
				.await?;

				info!("{} / {}", i + 1, data.len());
			}

			transaction.commit().await?;
			Ok(data.len())
		}
		InputKind::Zero(data) => {
			for (
				i,
				ZeroInput {
					name,
					mapper_steamid64,
					..
				},
			) in data.iter().enumerate()
			{
				let Ok(mapper_id) = mapper_steamid64.parse::<u64>() else {
					continue;
				};

				if mapper_id <= MAGIC_NUMBER {
					continue;
				}

				let mapper_id = mapper_id - MAGIC_NUMBER;

				sqlx::query(&format!(
					r#"
					UPDATE maps
					SET created_by = {mapper_id}
					WHERE name = "{name}"
					"#
				))
				.execute(&mut transaction)
				.await?;

				info!("{} / {}", i + 1, data.len());
			}

			transaction.commit().await?;
			Ok(data.len())
		}
	}
}
