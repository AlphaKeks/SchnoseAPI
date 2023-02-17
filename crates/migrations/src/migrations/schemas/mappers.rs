use {
	crate::MAGIC_NUMBER,
	color_eyre::Result as Eyre,
	log::info,
	serde::{Deserialize, Serialize},
	sqlx::{MySql, Pool},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
	pub name: String,
	pub mapper_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
	pub map_name: String,
	pub mapper_name: String,
	pub mapper_id: u32,
}

pub async fn update(data: &[Input], pool: &Pool<MySql>) -> Eyre<usize> {
	let mut transaction = pool.begin().await?;

	for (i, Input { name, mapper_id }) in data.iter().enumerate() {
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
