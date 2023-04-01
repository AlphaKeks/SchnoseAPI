use {
	axum::{
		error_handling::HandleErrorLayer,
		http::StatusCode,
		routing::{get, post},
		BoxError, Router,
	},
	clap::Parser,
	color_eyre::Result as Eyre,
	log::{debug, info},
	serde::{Deserialize, Serialize},
	sqlx::{mysql::MySqlPoolOptions, MySql, Pool},
	std::{net::SocketAddr, path::PathBuf, time::Duration},
	tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder},
};

mod ser_date;

mod models;
pub(crate) use models::{error::Error, Response, ResponseBody};

mod routes;

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;
	let args = Args::parse();

	let config_file = std::fs::read_to_string(args.config_path)?;
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
		.min_connections(30)
		.max_connections(100)
		.connect(&config.mysql_url)
		.await?;
	debug!("Connected to database.");

	let global_state = GlobalState { pool };

	/* TODO:
	 * `/records/top/world_records`
	 */
	let router = Router::new()
		.route("/", get(routes::index))
		.route("/api/", get(routes::index))
		.route("/api", get(routes::index))
		.route("/api/modes/:ident", get(routes::modes::ident))
		.route("/api/modes/", get(routes::modes::index))
		.route("/api/modes", get(routes::modes::index))
		.route("/api/players/:ident", get(routes::players::ident))
		.route("/api/players/", get(routes::players::index))
		.route("/api/players", get(routes::players::index))
		.route("/api/players/:ident/completion", get(routes::players::completion))
		.route("/api/servers/:ident", get(routes::servers::ident))
		.route("/api/servers/", get(routes::servers::index))
		.route("/api/servers", get(routes::servers::index))
		.route("/api/maps/:ident", get(routes::maps::ident))
		.route("/api/maps/", get(routes::maps::index))
		.route("/api/maps", get(routes::maps::index))
		.route("/api/maps/filters", get(routes::maps::filters))
		.route("/api/records/:id", get(routes::records::id))
		.route("/api/records/", get(routes::records::index))
		.route("/api/records", get(routes::records::index))
		// TODO: when filtering by mode, exclude courses that aren't possible
		.route("/api/records/top/player/:ident", get(routes::records::player))
		.route("/api/records/top/map/:ident", get(routes::records::map))
		// TODO: parameters?
		.route("/api/records/place/:id", get(routes::records::place))
		.route("/api/twitch_info", post(routes::twitch_info).layer(
			ServiceBuilder::new().layer(HandleErrorLayer::new(|why: BoxError| async move {
				(
					StatusCode::TOO_MANY_REQUESTS,
					why.to_string()
				)
			}))
			.layer(BufferLayer::new(1024))
			.layer(RateLimitLayer::new(5, Duration::from_secs(10)))
		))
		// .route("/api/twitch_info", post(routes::twitch_info))
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

	#[arg(long = "log")]
	#[clap(default_value = "INFO")]
	/// `RUST_LOG` level.
	log_level: String,

	#[arg(long = "config")]
	#[clap(default_value = "./config.toml")]
	/// Path to the config file containing the IP, port, log level and some secrets.
	config_path: PathBuf,
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
