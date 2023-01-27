use {
	color_eyre::Result as Eyre,
	gokz_rs::{prelude::Mode, records::Record},
	log::{debug, info},
	sqlx::{MySql, Pool},
};

pub(crate) async fn insert(
	input: Vec<Record>,
	chunk_size: u64,
	table_name: &str,
	database_connection: &Pool<MySql>,
) -> Eyre<()> {
	let total = input.len();
	debug!("> {} records", total);

	for (i, modes) in input.chunks(chunk_size as usize).enumerate() {
		let sql_query = build_query(modes, table_name).await;
		sqlx::query(&sql_query).execute(database_connection).await?;
		info!("{} / {} rows. ({}%)", i, total, (i as f32 / total as f32) * 100.0);
	}

	Ok(())
}

async fn build_query(records: &[Record], table_name: &str) -> String {
	let Record {
		id,
		steamid64,
		player_name: _,
		steam_id: _,
		server_id,
		map_id,
		stage,
		mode,
		tickrate: _,
		time,
		teleports,
		created_on,
		updated_on: _,
		updated_by: _,
		record_filter_id: _,
		server_name: _,
		map_name: _,
		points: _,
		replay_id: _,
	} = &records[0];

	let mode: Mode = mode.parse().expect("Encountered invalid mode");
	let mode_id = mode as u8;

	let mut query = format!(
		r#"
INSERT IGNORE INTO {table_name}
  (
    map_id,
    mode_id,
    player_id,
    server_id,
    stage,
    teleports,
    time,
    created_on,
    global_id
  )
VALUES
  (
    {map_id},
    {mode_id},
    {steamid64},
    {server_id},
    {stage},
    {teleports},
    {time},
    "{created_on}",
    {id}
  )"#
	);

	for Record {
		id,
		steamid64,
		player_name: _,
		steam_id: _,
		server_id,
		map_id,
		stage,
		mode: _,
		tickrate: _,
		time,
		teleports,
		created_on,
		updated_on: _,
		updated_by: _,
		record_filter_id: _,
		server_name: _,
		map_name: _,
		points: _,
		replay_id: _,
	} in records.iter().skip(1)
	{
		query.push_str(&format!(
			r#"
 ,(
    {map_id},
    {mode_id},
    {steamid64},
    {server_id},
    {stage},
    {teleports},
    {time},
    "{created_on}",
    {id}
  )"#
		));
	}

	query
}
