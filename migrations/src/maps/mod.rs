use {
	api_scraper::MergedMap,
	color_eyre::Result as Eyre,
	gokz_rs::{players::Player, prelude::*, GlobalAPI},
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

	let client = gokz_rs::Client::new();
	for (i, maps) in input.chunks(chunk_size as usize).enumerate() {
		let sql_query = build_query(maps, table_name, database_connection, &client).await?;
		sqlx::query(&sql_query).execute(database_connection).await?;
		info!("{} / {} rows. ({}%)", i, total, (i as f32 / total as f32) * 100.0);
	}

	Ok(())
}

async fn build_query(
	maps: &[MergedMap],
	table_name: &str,
	database_connection: &Pool<MySql>,
	client: &gokz_rs::Client,
) -> Eyre<String> {
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
INSERT IGNORE INTO {table_name}
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

	for (
		i,
		MergedMap {
			id,
			name,
			difficulty,
			validated,
			filesize,
			created_by,
			approved_by,
			created_on,
			updated_on,
		},
	) in maps.iter().skip(1).enumerate()
	{
		if (sqlx::query_as::<_, PlayerID>(&format!(
			"SELECT id FROM players WHERE id = {}",
			created_by
		))
		.fetch_one(database_connection)
		.await)
			.is_err()
		{
			let player = match GlobalAPI::get_player(
				&PlayerIdentifier::SteamID64(*created_by),
				client,
			)
			.await
			{
				Ok(player) => player,
				Err(why) => match why.kind {
					ErrorKind::Parsing { expected: _, got: _ } => Player {
						steamid64: created_by.to_string(),
						steam_id: SteamID::from(*created_by).to_string(),
						is_banned: false,
						total_records: 0,
						name: String::from("unknown"),
					},
					_ => panic!("`{created_by}`: FUCK {why:?}"),
				},
			};
			sqlx::query(&format!(
				r#"INSERT IGNORE INTO players (id, name, is_banned) VALUES ({}, "{}", {})"#,
				player.steamid64.parse::<u64>()?,
				player.name,
				player.is_banned
			))
			.execute(database_connection)
			.await?;
			info!("ResidentSleeper");
			std::thread::sleep(std::time::Duration::from_millis(1000));
		}

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
		info!("{} / {}", i + 1, maps.len());
	}

	Ok(query)
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, sqlx::FromRow)]
struct PlayerID {
	id: u64,
}
