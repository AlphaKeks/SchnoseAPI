#![warn(clippy::style, clippy::complexity, clippy::cognitive_complexity)]
#![deny(clippy::perf, clippy::correctness)]

use {
	axum::{routing::get, Router, Server},
	clap::Parser,
	color_eyre::Result,
	log::{debug, info},
	serde::Deserialize,
	sqlx::{mysql::MySqlPoolOptions, MySql, Pool},
	std::{net::SocketAddr, path::PathBuf},
};

#[derive(Debug, Parser)]
struct Args {
	/// Print debug information.
	#[arg(long)]
	#[clap(default_value = "false")]
	debug: bool,

	/// Print debug information.
	#[arg(short, long = "config")]
	#[clap(default_value = "./config.toml")]
	config_path: PathBuf,

	/// `RUST_LOG` value.
	#[arg(long = "log")]
	log_level: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Config {
	ip_address: [u8; 4],
	port: u16,
	database_url: String,
	log_level: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GlobalState {
	pub conn: Pool<MySql>,
}

mod routes;

pub use backend::{DatabaseError, Error};

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;
	let args = Args::parse();

	let config_file = std::fs::read_to_string(&args.config_path)?;
	let config: Config = toml::from_str(&config_file)?;

	std::env::set_var(
		"RUST_LOG",
		if args.debug {
			"DEBUG"
		} else if let Some(ref log_level) = args.log_level {
			log_level.as_str()
		} else if let Some(ref log_level) = config.log_level {
			log_level.as_str()
		} else {
			"backend=INFO"
		},
	);
	env_logger::init();

	debug!("{args:#?}");

	let addr = SocketAddr::from((config.ip_address, config.port));
	info!("Listening on {addr}.");

	let pool = MySqlPoolOptions::new()
		.min_connections(50)
		.max_connections(100)
		.connect(&config.database_url)
		.await?;
	info!("Connected to database.");

	let global_state = GlobalState { conn: pool };

	let router = Router::new()
		.route("/", get(|| async { "(͡ ͡° ͜ つ ͡͡°)" }))
		.route("/api/players", get(routes::players::get_index))
		.route("/api/players/", get(routes::players::get_index))
		.route("/api/players/:identifier", get(routes::players::get_by_identifier))
		.with_state(global_state);

	Server::bind(&addr)
		.serve(router.into_make_service())
		.await
		.expect("Failed to start server.");

	Ok(())
}
