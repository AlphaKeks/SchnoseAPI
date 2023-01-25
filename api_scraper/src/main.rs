use sqlx::mysql::MySqlPoolOptions;

mod maps;
mod output;

use {
	clap::{Parser, Subcommand, ValueEnum},
	color_eyre::Result as Eyre,
	serde::{Deserialize, Serialize},
};

/// CLI tool to continuously fetch data from the GlobalAPI
#[derive(Debug, Parser)]
struct Args {
	/// Which endpoint to target
	#[command(subcommand)]
	endpoint: Endpoint,

	/// Delay between each request in milliseconds. Defaults to `1000`.
	#[arg(long)]
	delay: Option<usize>,

	/// Output method. Defaults to `json`.
	#[arg(long)]
	output_method: OutputMethod,

	/// Path for the output if `json` was specified as the output method.
	#[arg(long)]
	output_path: Option<String>,

	/// MySQL table name if `mysql` was specified as the output method.
	#[arg(long)]
	table_name: Option<String>,

	/// `config.toml` file path with the MySQL connection string
	#[arg(short, long)]
	config_path: Option<String>,

	/// Don't print any output. The `debug` flag overrides this flag.
	#[arg(short, long)]
	quiet: bool,

	/// Print debug information.
	#[arg(short, long)]
	debug: bool,
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

	let connection = match args.output_method {
		OutputMethod::Json => None,
		OutputMethod::MySQL => {
			let config_path = args.config_path.unwrap_or_else(|| String::from("./config.toml"));
			let config_file = std::fs::read_to_string(&config_path)
				.unwrap_or_else(|_| panic!("Couldn't find config file at `{}`.", config_path));
			let config: Config =
				toml::from_str(&config_file).expect("Failed to parse `config.toml`");

			Some(
				MySqlPoolOptions::new()
					.max_connections(50)
					.connect(&config.database_url)
					.await?,
			)
		},
	};

	#[allow(unused)]
	match args.endpoint {
		Endpoint::Maps => {
			maps::fetch_maps(args.output_method, args.output_path, args.table_name, connection)
				.await?;
		},
		Endpoint::Modes => {
			todo!();
		},
		Endpoint::Players { start_id, offset, limit } => {
			todo!();
		},
		Endpoint::Records { start_id, limit } => {
			todo!();
		},
		Endpoint::Servers => {
			todo!();
		},
	}

	Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
	database_url: String,
}

#[derive(Debug, Clone, Copy, Subcommand)]
enum Endpoint {
	Maps,
	Modes,
	Players { start_id: usize, offset: usize, limit: Option<usize> },
	Records { start_id: usize, limit: Option<usize> },
	Servers,
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputMethod {
	Json,
	MySQL,
}
