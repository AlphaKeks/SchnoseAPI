mod models;
mod routes;

use {
	axum::{routing::get, Router},
	clap::Parser,
	color_eyre::Result as Eyre,
	log::info,
	serde::{Deserialize, Serialize},
	sqlx::{mysql::MySqlPoolOptions, MySql, Pool},
	std::net::SocketAddr,
};

use routes::{index, maps, modes, players, servers};

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;

	let args = Args::parse();

	std::env::set_var("RUST_LOG", "api=INFO");
	if args.debug {
		std::env::set_var("RUST_LOG", "api=DEBUG");
	}

	env_logger::init();

	let config_file =
		std::fs::read_to_string(args.config.unwrap_or_else(|| String::from("./config.toml")))
			.expect("Missing `config.toml` file.");
	let config: Config = toml::from_str(&config_file)?;

	let pool = MySqlPoolOptions::new()
		.min_connections(10)
		.max_connections(100)
		.connect(&config.database_url)
		.await?;

	let addr = {
		let ip = if args.public { [0, 0, 0, 0] } else { [127, 0, 0, 1] };

		SocketAddr::from((ip, args.port.unwrap_or(3000)))
	};

	info!("Listening on `{addr}`.");

	let app = Router::new()
		.route("/", get(index))
		.route("/api", get(index))
		.route("/api/", get(index))
		.route("/api/players/id/:id", get(players::id))
		.route("/api/players/name/:name", get(players::name))
		.route("/api/players", get(players::index))
		.route("/api/maps/id/:id", get(maps::id))
		.route("/api/maps/name/:name", get(maps::name))
		.route("/api/maps", get(maps::index))
		.route("/api/servers/id/:id", get(servers::id))
		.route("/api/servers/name/:name", get(servers::name))
		.route("/api/servers", get(servers::index))
		.route("/api/modes/id/:id", get(modes::id))
		.route("/api/modes/name/:name", get(modes::name))
		.with_state(GlobalState { pool });

	axum::Server::bind(&addr)
		.serve(app.into_make_service())
		.await
		.expect("Failed to start server.");

	Ok(())
}

#[derive(Debug, Parser)]
struct Args {
	/// Defaults to `3000`
	#[arg(short, long)]
	port: Option<u16>,

	/// The `config.toml` file containing the database connection string. Defaults to
	/// `./config.toml`
	#[arg(short, long)]
	config: Option<String>,

	/// Whether to open on `0.0.0.0` instead of `127.0.0.1`
	#[arg(long)]
	public: bool,

	/// Print debug information.
	#[arg(short, long)]
	debug: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
	database_url: String,
}

#[derive(Debug, Clone)]
pub struct GlobalState {
	pool: Pool<MySql>,
}
