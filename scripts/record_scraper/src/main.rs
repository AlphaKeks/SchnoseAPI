#![deny(clippy::perf)]
#![warn(clippy::suspicious, clippy::style)]

use gokz_rs::GlobalAPI;

use {
	chrono::{DateTime, NaiveDateTime, Utc},
	clap::{Parser, Subcommand},
	color_eyre::Result as Eyre,
	database::{
		crd::read::{get_map, get_server},
		schemas::steam_id64_to_account_id,
	},
	gokz_rs::{
		prelude::{Mode as GOKZMode, *},
		records::Record as GlobalRecord,
	},
	log::info,
	serde::{Deserialize, Serialize},
	sqlx::mysql::MySqlPoolOptions,
	std::{path::PathBuf, time::Instant},
};

#[derive(Debug, Parser)]
struct Args {
	/// What to do
	#[command(subcommand)]
	mode: Mode,

	/// Config file containing a MySQL connection string
	#[arg(short, long)]
	#[clap(default_value = "./config.toml")]
	config_file: PathBuf,

	/// Print debug information
	#[arg(long)]
	#[clap(default_value = "false")]
	debug: bool,
}

#[derive(Debug, Subcommand)]
enum Mode {
	/// Read records from a file and insert them into the database.
	InputFile { file: PathBuf },
	/// Scrape records from the GlobalAPI and insert them into the database.
	Scrape { start_id: u32 },
}

#[derive(Debug, Deserialize)]
struct Config {
	mysql_url: String,
}

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;
	let args = Args::parse();
	let config_file = std::fs::read_to_string(args.config_file)?;
	let config: Config = toml::from_str(&config_file)?;

	std::env::set_var("RUST_LOG", if args.debug { "DEBUG" } else { "record_scraper=INFO" });
	env_logger::init();

	let pool = MySqlPoolOptions::new()
		.connect(&config.mysql_url)
		.await?;

	let start = Instant::now();

	match args.mode {
		Mode::InputFile { file } => {
			let file = std::fs::read_to_string(file)?;

			match serde_json::from_str::<Vec<ElasticRecord>>(&file) {
				Ok(elastic_records) => {
					let mut records = Vec::new();
					for record in elastic_records {
						let map = get_map(MapIdentifier::Name(record.map_name), &pool).await?;
						let course_id = (map.id * 100) as u32 + record.stage as u32;
						let mode = record.mode.parse::<GOKZMode>()?;
						let steam_id64 = record.steamid64.parse::<u64>()?;
						let player_id = steam_id64_to_account_id(steam_id64)?;
						let server = get_server(
							record
								.server_name
								.replace([',', '\''], ""),
							&pool,
						)
						.await?;
						let created_on = DateTime::<Utc>::from_utc(
							NaiveDateTime::parse_from_str(&record.created_on, "%Y-%m-%dT%H:%M:%S")?,
							Utc,
						);

						if database::crd::read::get_record(record.id, &pool)
							.await
							.is_err()
						{
							records.push((
								record.id,
								course_id,
								mode as u8,
								player_id,
								server.id,
								record.time,
								record.teleports,
								created_on,
							));
						} else {
							info!("Skipping `{}`.", record.id);
						}
					}

					database::crd::create::insert_records(&records, &pool).await?;
				}
				_ => match serde_json::from_str::<Vec<GlobalRecord>>(&file) {
					Ok(global_records) => {
						let mut records = Vec::new();
						for record in global_records {
							let record_id = record.id as u32;
							let course_id = ((record.map_id * 100) + record.stage) as u32;
							let mode = record.mode.parse::<GOKZMode>()?;
							let mode_id = mode as u8;
							let steam_id64 = record.steamid64.parse::<u64>()?;
							let player_id = steam_id64_to_account_id(steam_id64)?;
							let server_id = record.server_id as u16;
							let time = record.time;
							let teleports = record.teleports as u32;
							let created_on = DateTime::<Utc>::from_utc(
								NaiveDateTime::parse_from_str(
									&record.created_on,
									"%Y-%m-%dT%H:%M:%S",
								)?,
								Utc,
							);

							if database::crd::read::get_record(record_id, &pool)
								.await
								.is_err()
							{
								records.push((
									record_id, course_id, mode_id, player_id, server_id, time,
									teleports, created_on,
								));
							} else {
								info!("Skipping `{record_id}`.");
							}
						}

						database::crd::create::insert_records(&records, &pool).await?;
					}
					_ => panic!("Invalid input format."),
				},
			}
		}
		Mode::Scrape { start_id } => {
			let client = gokz_rs::Client::new();

			for i in start_id.. {
				let record = loop {
					if let Ok(record) = GlobalAPI::get_record(i as i32, &client).await {
						break record;
					} else {
						info!("No new records...");
						continue;
					}
				};

				let record_id = record.id as u32;
				let course_id = ((record.map_id * 100) + record.stage) as u32;
				let mode = record.mode.parse::<GOKZMode>()?;
				let mode_id = mode as u8;
				let steam_id64 = record.steamid64.parse::<u64>()?;
				let player_id = steam_id64_to_account_id(steam_id64)?;
				let server_id = record.server_id as u16;
				let time = record.time;
				let teleports = record.teleports as u32;
				let created_on = DateTime::<Utc>::from_utc(
					NaiveDateTime::parse_from_str(&record.created_on, "%Y-%m-%dT%H:%M:%S")?,
					Utc,
				);

				// If the player isn't in DB, insert them before inserting the record.
				if database::crd::read::get_player(PlayerIdentifier::SteamID64(steam_id64), &pool)
					.await
					.is_err()
				{
					database::crd::create::insert_players(
						&[(
							player_id,
							record
								.player_name
								.unwrap_or_else(|| String::from("unknown")),
							0,
						)],
						&pool,
					)
					.await?;
				}

				database::crd::create::insert_records(
					&[(
						record_id, course_id, mode_id, player_id, server_id, time, teleports,
						created_on,
					)],
					&pool,
				)
				.await?;
				info!("Inserted record `{record_id}`.");
			}
		}
	};

	info!(
		"Finished after {:.3} seconds.",
		(Instant::now() - start).as_millis() as f64 / 1000.0
	);

	Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElasticRecord {
	mode: String,
	server_name: String,
	tickrate: u8,
	stage: u8,
	created_on: String,
	map_name: String,
	teleports: u32,
	id: u32,
	steamid64: String,
	time: f64,
	player_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseRecord {
	id: u32,
	course_id: u32,
	mode_id: u8,
	player_id: u32,
	server_id: u16,
	time: f64,
	teleports: u32,
	created_on: String,
}
