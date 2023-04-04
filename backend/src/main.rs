#![warn(clippy::style, clippy::complexity, clippy::cognitive_complexity)]
#![deny(clippy::perf, clippy::correctness)]

use {
	axum::{routing::get, Router, Server},
	clap::Parser,
	color_eyre::{eyre::eyre, Result},
	serde::Deserialize,
	sqlx::{mysql::MySqlPoolOptions, MySql, Pool},
	std::{net::SocketAddr, path::PathBuf},
	time::macros::format_description,
	tower_http::trace::TraceLayer,
	tracing::{debug, info, level_filters::LevelFilter},
	tracing_subscriber::fmt::time::UtcTime,
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

	tracing_subscriber::fmt()
		.compact()
		.with_line_number(true)
		.with_timer(UtcTime::new(format_description!(
			"[[[year]-[month]-[day] [hour]:[minute]:[second]]"
		)))
		.with_max_level(if args.debug {
			LevelFilter::DEBUG
		} else if let Some(ref log_level) = args.log_level {
			log_level
				.parse()
				.map_err(|_| eyre!("Log level is required!"))?
		} else if let Some(ref log_level) = config.log_level {
			log_level
				.parse()
				.map_err(|_| eyre!("Log level is required!"))?
		} else {
			LevelFilter::INFO
		})
		.init();

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
		.route("/api", get(|| async { "(͡ ͡° ͜ つ ͡͡°)" }))
		.route("/api/", get(|| async { "(͡ ͡° ͜ つ ͡͡°)" }))
		.route("/api/players", get(routes::players::get_index))
		.route("/api/players/", get(routes::players::get_index))
		.route("/api/players/:identifier", get(routes::players::get_by_identifier))
		.route("/api/modes", get(routes::modes::get_index))
		.route("/api/modes/", get(routes::modes::get_index))
		.route("/api/modes/:identifier", get(routes::modes::get_by_identifier))
		.route("/api/maps", get(routes::maps::get_index))
		.route("/api/maps/", get(routes::maps::get_index))
		.route("/api/maps/:identifier", get(routes::maps::get_by_identifier))
		.route("/api/servers", get(routes::servers::get_index))
		.route("/api/servers/", get(routes::servers::get_index))
		.route("/api/servers/:identifier", get(routes::servers::get_by_identifier))
		.route("/api/records/:id", get(routes::records::get_by_id))
		.with_state(global_state)
		.layer(TraceLayer::new_for_http());

	Server::bind(&addr)
		.serve(router.into_make_service())
		.await
		.expect("Failed to start server.");

	Ok(())
}
