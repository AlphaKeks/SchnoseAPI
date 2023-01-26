use {
	color_eyre::Result as Eyre,
	gokz_rs::players::Player,
	log::{debug, info},
	sqlx::{MySql, Pool},
};

pub(crate) async fn insert(
	input: Vec<Player>,
	chunk_size: u64,
	table_name: &str,
	database_connection: &Pool<MySql>,
) -> Eyre<()> {
	let total = input.len();
	debug!("> {} players", total);

	for (i, modes) in input.chunks(chunk_size as usize).enumerate() {
		let sql_query = build_query(modes, table_name);
		sqlx::query(&sql_query).execute(database_connection).await?;
		info!("{} / {} rows. ({}%)", i, total, (i as f32 / total as f32) * 100.0);
	}

	Ok(())
}

fn build_query(players: &[Player], table_name: &str) -> String {
	let Player { steamid64, steam_id: _, is_banned, total_records: _, name } = &players[0];

	let mut query = format!(
		r#"
INSERT INTO {table_name}
  (
    id,
    name,
    is_banned
  )
VALUES
  (
    {steamid64},
    "{name}",
    {is_banned}
  )"#
	);

	for Player { steamid64, steam_id: _, is_banned, total_records: _, name } in
		players.iter().skip(1)
	{
		query.push_str(&format!(
			r#"
 ,(
    {steamid64},
    "{name}",
    {is_banned}
  )"#
		));
	}

	query
}
