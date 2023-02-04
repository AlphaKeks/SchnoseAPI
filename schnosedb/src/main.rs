mod migrations;

use {
	clap::Parser,
	color_eyre::Result as Eyre,
	gokz_rs::{maps::Map, modes::APIMode, players::Player, records::Record, servers::Server},
	log::info,
	migrations::{
		schemas::{
			self, maps::MapSchema, modes::ModeSchema, players::PlayerSchema, records::RecordSchema,
			servers::ServerSchema,
		},
		Schema, SqlAction,
	},
	serde::{Deserialize, Serialize},
	sqlx::mysql::MySqlPoolOptions,
	std::path::PathBuf,
};

// don't ask
pub const MAGIC_NUMBER: u64 = 76561197960265728;

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;
	let args = Args::parse();

	std::env::set_var("RUST_LOG", if args.debug { "schnosedb=DEBUG" } else { "schnosedb=INFO" });
	env_logger::init();

	let config_file =
		std::fs::read_to_string(args.config_path).expect("Failed to locate `config.toml`.");
	let config: Config = toml::from_str(&config_file)?;

	let pool = MySqlPoolOptions::new()
		.min_connections(10)
		.max_connections(100)
		.connect(&config.mysql_url)
		.await?;

	let gokz_client = gokz_rs::Client::new();

	match args.action {
		SqlAction::Up => migrations::up(&pool).await?,
		SqlAction::Down => migrations::down(&pool).await?,
		SqlAction::Redo => {
			migrations::up(&pool).await?;
			migrations::down(&pool).await?;
		}
		SqlAction::Insert { schema, data } => match schema {
			Schema::Players => {
				let data = std::fs::read_to_string(data)?;
				let data = serde_json::from_str::<Vec<Player>>(&data)?
					.into_iter()
					.filter_map(|player| PlayerSchema::try_from(player).ok())
					.collect::<Vec<PlayerSchema>>();
				let count = schemas::players::insert(&data, &pool).await?;
				info!("Inserted {count} rows into `players`.");
			}
			Schema::Modes => {
				let data = std::fs::read_to_string(data)?;
				let data = serde_json::from_str::<Vec<APIMode>>(&data)?
					.into_iter()
					.filter_map(|mode| ModeSchema::try_from(mode).ok())
					.collect::<Vec<ModeSchema>>();
				let count = schemas::modes::insert(&data, &pool).await?;
				info!("Inserted {count} rows into `modes`.");
			}
			Schema::Servers => {
				let data = std::fs::read_to_string(data)?;
				let data = serde_json::from_str::<Vec<Server>>(&data)?
					.into_iter()
					.filter_map(|server| ServerSchema::try_from(server).ok())
					.collect::<Vec<ServerSchema>>();
				let count = schemas::servers::insert(&data, &pool).await?;
				info!("Inserted {count} rows into `servers`.");
			}
			Schema::Maps => {
				let data = std::fs::read_to_string(data)?;
				let data = serde_json::from_str::<Vec<Map>>(&data)?
					.into_iter()
					.filter_map(|map| MapSchema::try_from(map).ok())
					.collect::<Vec<MapSchema>>();
				let kzgo_maps = gokz_rs::kzgo::KZGO::get_maps(&gokz_client).await?;
				let count = schemas::maps::insert(&data, kzgo_maps, &pool).await?;
				info!("Inserted {count} rows into `servers`.");
			}
			Schema::Courses => {
				let data = std::fs::read_to_string(data)?;
				let global_maps = serde_json::from_str::<Vec<Map>>(&data)?;
				let kzgo_maps = gokz_rs::kzgo::KZGO::get_maps(&gokz_client).await?;
				let count = schemas::courses::insert(global_maps, kzgo_maps, &pool).await?;
				info!("Inserted {count} rows into `courses`.");
			}
			Schema::Records => {
				let data = std::fs::read_to_string(data)?;
				let data = serde_json::from_str::<Vec<Record>>(&data)?
					.into_iter()
					.filter_map(|record| RecordSchema::try_from(record).ok())
					.collect::<Vec<RecordSchema>>();
				let count = schemas::records::insert(&data, &pool).await?;
				info!("Inserted {count} rows into `records`.");
			}
		},
	}

	Ok(())
}

#[derive(Debug, Parser)]
struct Args {
	/// What do?
	#[command(subcommand)]
	action: SqlAction,

	/// Path to `config.toml`
	#[arg(short, long)]
	#[clap(default_value = "./config.toml")]
	config_path: PathBuf,

	/// Print debug output.
	#[arg(long)]
	debug: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
	mysql_url: String,
}
