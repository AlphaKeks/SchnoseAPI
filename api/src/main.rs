use {
	axum::{routing::get, Router},
	clap::Parser,
	color_eyre::Result as Eyre,
	log::{debug, info},
	serde::{Deserialize, Serialize},
	sqlx::{mysql::MySqlPoolOptions, MySql, Pool},
	std::{net::SocketAddr, path::PathBuf},
};

mod models;
pub(crate) use models::error::Error;

mod routes;

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;
	let args = Args::parse();

	let config_file = std::fs::read_to_string(args.config_file)?;
	let config: Config = toml::from_str(&config_file)?;

	std::env::set_var(
		"RUST_LOG",
		if args.debug {
			String::from("DEBUG")
		} else if let Some(level) = config.log_level {
			level
		} else {
			args.log_level
		},
	);
	env_logger::init();

	let addr = SocketAddr::from((config.address, config.port));
	info!("Listening on {addr}.");

	let pool = MySqlPoolOptions::new()
		.min_connections(10)
		.max_connections(50)
		.connect(&config.mysql_url)
		.await?;
	debug!("Connected to database.");

	let global_state = GlobalState { pool };

	let router = Router::new()
		.route("/", get(routes::index))
		.route("/api/modes/:ident", get(routes::modes::ident))
		.route("/api/modes/", get(routes::modes::index))
		.route("/api/modes", get(routes::modes::index))
		.route("/api/players/:ident", get(routes::players::ident))
		.route("/api/players/", get(routes::players::index))
		.route("/api/players", get(routes::players::index))
		.route("/api/servers/:ident", get(routes::servers::ident))
		.route("/api/servers/", get(routes::servers::index))
		.route("/api/servers", get(routes::servers::index))
		.route("/api/maps/:ident", get(routes::maps::ident))
		.route("/api/maps/", get(routes::maps::index))
		.route("/api/maps", get(routes::maps::index))
		.with_state(global_state);

	axum::Server::bind(&addr)
		.serve(router.into_make_service())
		.await
		.expect("Failed to run server.");

	Ok(())
}

#[derive(Debug, Parser)]
struct Args {
	#[arg(long)]
	#[clap(default_value = "false")]
	/// Print debug information.
	debug: bool,

	#[arg(long)]
	#[clap(default_value = "INFO")]
	/// `RUST_LOG`
	log_level: String,

	#[arg(long)]
	#[clap(default_value = "./config.toml")]
	/// Config file
	config_file: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
	address: [u8; 4],
	port: u16,
	mysql_url: String,
	log_level: Option<String>,
}

#[derive(Clone)]
pub(crate) struct GlobalState {
	pub(crate) pool: Pool<MySql>,
}
