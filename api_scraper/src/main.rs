mod maps;
mod modes;
mod output;
mod players;
mod records;
mod servers;

use {
	clap::{Parser, Subcommand, ValueEnum},
	color_eyre::Result as Eyre,
};

/// CLI tool to continuously fetch data from the GlobalAPI
#[derive(Debug, Parser)]

struct Args {
	/// Path for the output json file.
	#[arg(short, long)]
	output_path: Option<String>,

	/// Delay between each request in milliseconds. Defaults to `1000`.
	#[arg(short, long)]
	delay: Option<u64>,

	/// Don't print any output. The `debug` flag overrides this flag.
	#[arg(short, long)]
	quiet: bool,

	/// Print debug information.
	#[arg(long)]
	debug: bool,

	/// Which endpoint to target.
	#[command(subcommand)]
	endpoint: Endpoint,
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

	match args.endpoint {
		Endpoint::Maps => {
			maps::fetch_maps(args.output_path).await?;
		},
		Endpoint::Modes => {
			modes::fetch_modes(args.output_path).await?;
		},
		Endpoint::Players { start_offset, chunk_size, backwards, limit } => {
			let delay = args.delay.unwrap_or(1000);
			let chunk_size = chunk_size.unwrap_or(1);

			players::fetch_players(
				start_offset,
				chunk_size,
				backwards.unwrap_or(false),
				limit.unwrap_or(chunk_size),
				delay,
				args.output_path,
			)
			.await?;
		},
		Endpoint::Records { start_id, limit, backwards } => {
			let delay = args.delay.unwrap_or(1000);

			records::fetch_records(
				start_id,
				backwards.unwrap_or(false),
				limit.unwrap_or(1),
				delay,
				args.output_path,
			)
			.await?;
		},
		Endpoint::Servers => {
			servers::fetch_servers(args.output_path).await?;
		},
	}

	Ok(())
}

#[derive(Debug, Clone, Copy, Subcommand)]
enum Endpoint {
	/// Fetch maps from the GlobalAPI.
	Maps,
	/// Fetch modes from the GlobalAPI.
	Modes,
	/// Fetch players from the GlobalAPI.
	Players {
		/// The offset at which to start scraping players
		start_offset: i32,
		/// How many players you want per request. The maximum is 500.
		chunk_size: Option<u32>,
		/// Whether to scrape backwards or not.
		backwards: Option<bool>,
		/// How many players to fetch before stopping automatically.
		limit: Option<u32>,
	},
	/// Fetch records from the GlobalAPI.
	Records { start_id: isize, limit: Option<u32>, backwards: Option<bool> },
	/// Fetch servers from the GlobalAPI.
	Servers,
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputMethod {
	Json,
	MySQL,
}
