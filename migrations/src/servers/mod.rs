use {
	color_eyre::Result as Eyre,
	gokz_rs::{players::Player, prelude::*, servers::Server, GlobalAPI},
	log::{debug, info},
	sqlx::{MySql, Pool},
};

/// HACK: I don't actually know who approved the servers, and since this is a toy application
/// anyway I'm putting gosh here because he's very handsome
const BACKUP_SERVER_APPROVER: u64 = 76_561_198_292_029_626;

pub(crate) async fn insert(
	input: Vec<Server>,
	chunk_size: u64,
	table_name: &str,
	database_connection: &Pool<MySql>,
) -> Eyre<()> {
	let total = input.len();
	debug!("> {} servers", total);

	let client = gokz_rs::Client::new();
	for (i, modes) in input.chunks(chunk_size as usize).enumerate() {
		let sql_query = build_query(modes, table_name, database_connection, &client).await?;
		sqlx::query(&sql_query).execute(database_connection).await?;
		info!("{} / {} rows. ({}%)", i, total, (i as f32 / total as f32) * 100.0);
	}

	Ok(())
}

async fn build_query(
	servers: &[Server],
	table_name: &str,
	database_connection: &Pool<MySql>,
	client: &gokz_rs::Client,
) -> Eyre<String> {
	let Server { id, port: _, ip: _, name, owner_steamid64 } = &servers[0];

	let mut query = format!(
		r#"
INSERT INTO {table_name}
  (
    id,
    name,
    owner_id,
    approved_by
  )
VALUES
  (
    {id},
    "{}",
    {owner_steamid64},
    {}
  )"#,
		name.replace(['"', '\'', ',', '\\'], ""),
		BACKUP_SERVER_APPROVER
	);

	for (i, Server { id, port: _, ip: _, name, owner_steamid64 }) in
		servers.iter().skip(1).enumerate()
	{
		if (sqlx::query_as::<_, ServerID>(&format!(
			"SELECT id FROM players WHERE id = {}",
			owner_steamid64.parse::<u64>()?
		))
		.fetch_one(database_connection)
		.await)
			.is_err()
		{
			let player = match GlobalAPI::get_player(
				&PlayerIdentifier::SteamID64(owner_steamid64.parse::<u64>()?),
				client,
			)
			.await
			{
				Ok(player) => player,
				Err(why) => match why.kind {
					ErrorKind::Parsing { expected: _, got: _ } => Player {
						steamid64: owner_steamid64.to_owned(),
						steam_id: SteamID::from(owner_steamid64.parse::<u64>()?).to_string(),
						is_banned: false,
						total_records: 0,
						name: String::from("unknown"),
					},
					_ => panic!("`{owner_steamid64}`: FUCK {why:?}"),
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
    "{}",
    {},
    {}
  )"#,
			name.replace(['"', '\'', ',', '\\'], ""),
			owner_steamid64.parse::<u64>()?,
			BACKUP_SERVER_APPROVER
		));
		info!("{} / {}", i + 1, servers.len());
	}

	Ok(query)
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, sqlx::FromRow)]
struct ServerID {
	id: u64,
}
