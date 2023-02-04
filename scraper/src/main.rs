#![deny(clippy::complexity, clippy::perf)]
#![warn(clippy::suspicious, clippy::style)]

use {
	chrono::{TimeZone, Utc},
	clap::{Parser, Subcommand},
	color_eyre::{eyre::eyre, Result as Eyre},
	gokz_rs::{prelude::*, GlobalAPI},
	log::{debug, error, info},
	serde::{Deserialize, Serialize},
	sqlx::{mysql::MySqlPoolOptions, FromRow},
	std::path::PathBuf,
};

pub const MAGIC_NUMBER: u64 = 76561197960265728;

#[derive(Debug, Parser)]
struct Args {
	/// MySQL table to insert into.
	#[arg(long)]
	table: String,

	/// Path to `config.toml`
	#[arg(short, long)]
	#[clap(default_value = "./config.toml")]
	config_path: PathBuf,

	/// Print no logs to stdout.
	#[arg(short, long)]
	#[clap(default_value = "false")]
	quiet: bool,

	/// Print debug information. This option overrides `quiet`.
	#[arg(long)]
	#[clap(default_value = "false")]
	debug: bool,

	/// Which data to fetch
	#[command(subcommand)]
	endpoint: Endpoint,
}

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;
	let args = Args::parse();
	let config_file = std::fs::read_to_string(args.config_path)?;
	let config: Config = toml::from_str(&config_file)?;

	// setup logging
	std::env::set_var("RUST_LOG", if args.quiet { "global_api=ERROR" } else { "global_api=INFO" });
	if args.debug {
		std::env::set_var("RUST_LOG", "global_api=DEBUG");
	}
	env_logger::init();

	let pool = MySqlPoolOptions::new()
		.connect(&config.mysql_url)
		.await?;

	match args.endpoint {
		Endpoint::Players { start_offset: mut offset } => {
			let mut total = 0;
			let gokz_client = gokz_rs::Client::new();

			offset -= 1;
			loop {
				offset += 1;
				let Ok(mut players) = GlobalAPI::get_players(Some(offset as i32), Some(1), &gokz_client).await else {
					error!("Failed to get player `{offset}`.");
					continue;
				};

				let player = players.remove(0);
				debug!("player `{offset}`: {player:?}");

				let Ok(steamid) = player.steamid64.parse::<u64>() else {
					error!("Failed to parse `{}` into a steamid64.", player.steamid64);
					continue;
				};

				let Ok(name) = sanitize(player.name) else {
					error!("Bad player name");
					continue;
				};

				if let Err(why) = sqlx::query(&format!(
					r#"
					INSERT INTO players
					  (id, name, is_banned)
					VALUES
					  ({}, "{}", {})
					"#,
					steamid - MAGIC_NUMBER,
					name,
					player.is_banned
				))
				.execute(&pool)
				.await
				{
					error!("Failed to insert `{offset}` into db: {why:?}");
					continue;
				}

				total += 1;
				info!("{total} total players");
			}
		}
		Endpoint::Records { start_id: mut id } => {
			let mut total = 0;
			let gokz_client = gokz_rs::Client::new();

			let mut global_maps = GlobalAPI::get_maps(true, Some(9999), &gokz_client).await?;
			let non_global_maps = GlobalAPI::get_maps(true, Some(9999), &gokz_client).await?;
			global_maps.extend(non_global_maps);

			id -= 1;
			loop {
				id += 1;
				let Ok(record) = GlobalAPI::get_record(id as i32, &gokz_client).await else {
					error!("Failed to get record `{id}`.");
					continue;
				};

				debug!("record `{id}`: {record:?}");

				let Some(map_name) = record.map_name else {
					error!("record `{id}` has a cringe map name smh");
					continue;
				};

				let Some(map_id) = global_maps.iter().find_map(|map| {
					if map.name.eq(&map_name) {
						Some(map.id)
					} else {
						None
					}
				}) else {
					error!("record `{id}` has an invalid map name? ({map_name})");
					continue;
				};

				let Ok(CourseID(course_id)) = sqlx::query_as::<_, CourseID>(&format!("SELECT id FROM courses WHERE map_id = {map_id}")).fetch_one(&pool).await else {
					error!("couldn't find map `{map_id}` in database???");
					continue;
				};

				let mode_id = {
					let Ok(mode) = record.mode.parse::<Mode>() else {
					error!("record `{id}` has an invalid mode??? ({})", record.mode);
					continue;
					};

					mode as u8
				};

				let player_id = {
					let Ok(steamid64) = record.steamid64.parse::<u64>() else {
						error!("record `{id}` has an invalid steamid64 ({})", record.steamid64);
						continue;
					};

					steamid64 - MAGIC_NUMBER
				};

				let Ok(created_on) = Utc.datetime_from_str(&record.created_on, "%Y-%m-%dT%H:%M:%S") else {
					error!("record `{id}` has a bad date ({})", record.created_on);
					continue;
    			};

				if let Err(why) = sqlx::query(&format!(
					r#"
					INSERT INTO records
					  (id, course_id, mode_id, player_id, server_id, time, teleports, created_on)
					VALUES
					  ({}, {}, {}, {}, {}, {}, {}, "{}")
					"#,
					record.id,
					course_id + record.stage as u16,
					mode_id,
					player_id,
					record.server_id,
					record.time,
					record.teleports,
					created_on
				))
				.execute(&pool)
				.await
				{
					error!("Failed to insert `{id}` into db: {why:?}");
					continue;
				}

				total += 1;
				info!("{total} total records");
			}
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
	mysql_url: String,
}

#[derive(Debug, Clone, Copy, Subcommand)]
enum Endpoint {
	/// `/players`
	Players {
		#[arg(long = "offset")]
		#[clap(default_value = "0")]
		start_offset: usize,
	},

	/// `/records`
	Records {
		/// The ID to start at.
		#[arg(long = "start")]
		#[clap(default_value = "0")]
		start_id: usize,
	},
}

fn sanitize(input: String) -> Eyre<String> {
	if input == "Bad Steamid64" {
		Err(eyre!("bad player name"))
	} else {
		Ok(input.replace(['\'', '"', ',', '\\'], ""))
	}
}

#[derive(FromRow)]
struct CourseID(u16);
