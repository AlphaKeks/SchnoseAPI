mod maps;
mod modes;
mod players;
mod records;
mod servers;

use {
	api_scraper::MergedMap,
	clap::Parser,
	color_eyre::Result as Eyre,
	gokz_rs::{modes::APIMode, players::Player, records::Record, servers::Server},
	serde::{Deserialize, Serialize},
	sqlx::mysql::MySqlPoolOptions,
};

/// CLI tool to parse JSON data from the GlobalAPI and insert that data into a MySQL database.
#[derive(Debug, Parser)]

struct Args {
	/// JSON input to insert into the database. The data must be an array.
	#[arg(short, long)]
	input_path: String,

	/// MySQL table name to insert into.
	#[arg(short, long)]
	table_name: String,

	/// How many rows to insert at once. Defaults to `1000`.
	#[arg(short, long)]
	chunk_size: Option<u64>,

	/// Path to the `config.toml` file containing the database connection string. See
	/// `config.toml.example`. Defaults to `./config.toml`.
	#[arg(long = "config")]
	config_path: Option<String>,

	/// Don't print any output. The `debug` flag overrides this flag.
	#[arg(short, long)]
	quiet: bool,

	/// Print debug information.
	#[arg(long)]
	debug: bool,
}

const DEFAULT_CUNK_SIZE: u64 = 1000;
const DEFAULT_CONFIG_PATH: &str = "./config.toml";

macro_rules! parse {
	($i:expr => $t:ty) => {
		serde_json::from_str::<$t>($i)
	};
}

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;
	let args = Args::parse();

	std::env::set_var("RUST_LOG", "api_scraper=ERROR");

	if !args.quiet {
		std::env::set_var("RUST_LOG", "api_scraper=INFO");
	}

	if args.debug {
		std::env::set_var("RUST_LOG", "api_scraper=DEBUG");
	}

	env_logger::init();

	let chunk_size = args.chunk_size.unwrap_or(DEFAULT_CUNK_SIZE);

	let config_path = args.config_path.unwrap_or_else(|| String::from(DEFAULT_CONFIG_PATH));
	let config_file = std::fs::read_to_string(&config_path)?;
	let config: Config = toml::from_str(&config_file)?;
	let conn = MySqlPoolOptions::new().max_connections(1).connect(&config.database_url).await?;
	let table_name = &args.table_name;

	let input = std::fs::read_to_string(&args.input_path)?;

	match parse!(&input => MapInput) {
		Ok(m) => maps::insert(m, chunk_size, table_name, &conn).await?,
		_ => match parse!(&input => ModeInput) {
			Ok(m) => modes::insert(m, chunk_size, table_name, &conn).await?,
			_ => match parse!(&input => PlayerInput) {
				Ok(p) => players::insert(p, chunk_size, table_name, &conn).await?,
				_ => match parse!(&input => RecordInput) {
					Ok(r) => records::insert(r, chunk_size, table_name, &conn).await?,
					_ => match parse!(&input => ServerInput) {
						Ok(s) => servers::insert(s, chunk_size, table_name, &conn).await?,
						_ => panic!("Invalid JSON input."),
					},
				},
			},
		},
	}

	Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
	database_url: String,
}

pub(crate) type MapInput = Vec<MergedMap>;
pub(crate) type ModeInput = Vec<APIMode>;
pub(crate) type PlayerInput = Vec<Player>;
pub(crate) type RecordInput = Vec<Record>;
pub(crate) type ServerInput = Vec<Server>;
