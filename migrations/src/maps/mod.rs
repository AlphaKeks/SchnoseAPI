use {
	api_scraper::MergedMap,
	color_eyre::Result as Eyre,
	log::{debug, info},
	sqlx::{MySql, Pool},
};

pub(crate) async fn insert(
	input: Vec<MergedMap>,
	chunk_size: u64,
	table_name: &str,
	database_connection: &Pool<MySql>,
) -> Eyre<()> {
	let total = input.len();
	debug!("> {} maps", total);

	for (i, maps) in input.chunks(chunk_size as usize).enumerate() {
		let sql_query = build_query(maps, table_name);
		sqlx::query(&sql_query).execute(database_connection).await?;
		info!("{} / {} rows. ({}%)", i, total, (i as f32 / total as f32) * 100.0);
	}

	Ok(())
}

fn build_query(maps: &[MergedMap], table_name: &str) -> String {
	let MergedMap {
		id,
		name,
		difficulty,
		validated,
		filesize,
		created_by,
		approved_by,
		created_on,
		updated_on,
	} = &maps[0];

	let mut query = format!(
		r#"
INSERT INTO {table_name}
  (
    id,
    name,
    difficulty,
    validated,
    filesize,
    created_by,
    approved_by,
    created_on,
    updated_on
  )
VALUES
  (
    {id},
    "{name}",
    {difficulty},
    {validated},
    {filesize},
    {created_by},
    {approved_by},
    "{created_on}",
    "{updated_on}"
  )"#
	);

	for MergedMap {
		id,
		name,
		difficulty,
		validated,
		filesize,
		created_by,
		approved_by,
		created_on,
		updated_on,
	} in maps.iter().skip(1)
	{
		query.push_str(&format!(
			r#"
 ,(
    {id},
    "{name}",
    {difficulty},
    {validated},
    {filesize},
    {created_by},
    {approved_by},
    "{created_on}",
    "{updated_on}"
  )"#
		));
	}

	query
}
