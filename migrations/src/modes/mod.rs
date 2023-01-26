use {
	color_eyre::Result as Eyre,
	gokz_rs::{modes::APIMode, prelude::Mode},
	log::{debug, info},
	sqlx::{MySql, Pool},
};

pub(crate) async fn insert(
	input: Vec<APIMode>,
	chunk_size: u64,
	table_name: &str,
	database_connection: &Pool<MySql>,
) -> Eyre<()> {
	let total = input.len();
	debug!("> {} modes", total);

	for (i, modes) in input.chunks(chunk_size as usize).enumerate() {
		let sql_query = build_query(modes, table_name);
		sqlx::query(&sql_query).execute(database_connection).await?;
		info!("{} / {} rows. ({}%)", i, total, (i as f32 / total as f32) * 100.0);
	}

	Ok(())
}

fn build_query(modes: &[APIMode], table_name: &str) -> String {
	let APIMode {
		id,
		name,
		description: _,
		latest_version: _,
		latest_version_description: _,
		website: _,
		repo: _,
		contact_steamid64: _,
		supported_tickrates: _,
		created_on: _,
		updated_on,
		updated_by_id: _,
	} = &modes[0];

	let mode = Mode::try_from(*id as u8).expect("Encountered invalid mode id");
	let name_short = mode.short();
	let name_long = mode.to_string();

	let mut query = format!(
		r#"
INSERT INTO {table_name}
  (
    id,
    name,
    name_short,
    name_long,
    created_on
  )
VALUES
  (
    {id},
    "{name}",
    "{name_short}",
    "{name_long}",
    "{updated_on}"
  )"#
	);

	for APIMode {
		id,
		name,
		description: _,
		latest_version: _,
		latest_version_description: _,
		website: _,
		repo: _,
		contact_steamid64: _,
		supported_tickrates: _,
		created_on: _,
		updated_on,
		updated_by_id: _,
	} in modes.iter().skip(1)
	{
		let mode = Mode::try_from(*id as u8).expect("Encountered invalid mode id");
		let name_short = mode.short();
		let name_long = mode.to_string();

		query.push_str(&format!(
			r#"
 ,(
    {id},
    "{name}",
    "{name_short}",
    "{name_long}",
    "{updated_on}"
  )"#
		));
	}

	query
}
