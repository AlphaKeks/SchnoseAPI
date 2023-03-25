use {
	clap::{Parser, Subcommand},
	color_eyre::Result as Eyre,
	gokz_rs::{global_api, MapIdentifier, Mode, ServerIdentifier, SteamID},
	log::{info, warn},
	serde::Deserialize,
	sqlx::{
		mysql::MySqlPoolOptions,
		types::chrono::{DateTime, NaiveDateTime, Utc},
	},
	std::path::PathBuf,
	tokio::time::{sleep, Duration},
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

	/// What to do.
	#[command(subcommand)]
	mode: ExecutionMode,
}

#[derive(Debug, Subcommand)]
enum ExecutionMode {
	/// Scrape records from the GlobalAPI.
	Scrape {
		/// The ID to start with. If not specified, this will be retrieved from the database.
		start_id: Option<u32>,
	},

	/// Process a Json file containing an array of GlobalAPI records.
	Json {
		/// Path to the Json file.
		path: PathBuf,
	},
}

#[derive(Debug, Deserialize)]
struct Config {
	database_url: String,
}

/// Time to sleep between each request in milliseconds.
const SLEEP_DURATION: u64 = 700;

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;
	let args = Args::parse();

	std::env::set_var("RUST_LOG", if args.debug { "DEBUG" } else { "scraper=INFO" });
	env_logger::init();

	let config_file = std::fs::read_to_string(args.config_path)?;
	let config: Config = toml::from_str(&config_file)?;

	let pool = MySqlPoolOptions::new()
		.connect(&config.database_url)
		.await?;

	let gokz_client = gokz_rs::Client::new();

	match args.mode {
		ExecutionMode::Scrape { start_id } => {
			let start_id = match start_id {
				Some(id) => id,
				None => sqlx::query!("SELECT MAX(id) AS id FROM records")
					.fetch_one(&pool)
					.await?
					.id
					.map_or(0, |id| id + 1),
			};

			for current_id in start_id.. {
				// Don't DDoS the API.
				sleep(Duration::from_millis(SLEEP_DURATION)).await;

				let record = loop {
					if let Ok(record) = global_api::get_record(current_id, &gokz_client).await {
						break record;
					} else {
						info!("No new records. Sleeping for {SLEEP_DURATION}ms.");
						sleep(Duration::from_millis(SLEEP_DURATION)).await;
						continue;
					}
				};

				let course_id =
					database::select::get_course_by_map(MapIdentifier::ID(record.map_id), &pool)
						.await?
						.id;

				let player_id = if let Ok(player) =
					database::select::get_player(record.steam_id.into(), &pool).await
				{
					// Update player name if necessary.
					if player.name != record.player_name {
						info!(
							r#""{}" ({}) has changed their name to "{}"."#,
							player.name, record.steam_id, record.player_name
						);
						sqlx::query!(
							"UPDATE players SET name = ? WHERE id = ?",
							record.player_name,
							player.id
						)
						.execute(&pool)
						.await?;
					}

					// If a new record has a player but the player is marked as `banned` in the
					// database, that must mean the player got unbanned.
					if player.is_banned {
						info!(
							r#""{}" ({}) got unbanned, apparently."#,
							player.name, record.steam_id
						);
						sqlx::query!("UPDATE players SET is_banned = 0 WHERE id = ?", player.id)
							.execute(&pool)
							.await?;
					}

					player.id
				} else {
					info!(r#"Missing player "{}" ({:?})."#, record.player_name, record.steam_id);
					info!("Fetching...");

					let row = global_api::get_player(record.steam_id.into(), &gokz_client)
						.await
						.map_or(
							database::schemas::PlayerRow {
								id: record.steam_id.as_id32(),
								name: String::from("unknown"),
								is_banned: false,
							},
							|player| database::schemas::PlayerRow {
								id: player.steam_id.as_id32(),
								name: player.name,
								is_banned: player.is_banned,
							},
						);

					let player_id = row.id;

					database::insert::players(&[row], &pool).await?;

					player_id
				};

				let server_id = if let Ok(server) =
					database::select::get_server(record.server_id.into(), &pool).await
				{
					// Update server name if necessary.
					if server.name != record.server_name {
						info!(
							r#"Server "{}" ({}) has changed its name to "{}"."#,
							server.name, server.id, record.server_name
						);
						sqlx::query!(
							"UPDATE servers SET name = ? WHERE id = ?",
							record.server_name,
							server.id
						)
						.execute(&pool)
						.await?;
					}

					server.id
				} else {
					info!(r#"Missing server "{}" ({})."#, record.server_name, record.server_id);
					info!("Fetching...");

					let (row, owner_steamid) = global_api::get_server(
						&ServerIdentifier::ID(record.server_id),
						&gokz_client,
					)
					.await
					.map(|server| {
						(
							database::schemas::ServerRow {
								id: server.id,
								name: server.name,
								owned_by: server.owner_steamid.as_id32(),
								approved_by: 0,
							},
							server.owner_steamid,
						)
					})?;

					if database::select::get_player(owner_steamid.into(), &pool)
						.await
						.is_err()
					{
						let row = global_api::get_player(owner_steamid.into(), &gokz_client)
							.await
							.map_or(
								database::schemas::PlayerRow {
									id: owner_steamid.as_id32(),
									name: String::from("unknown"),
									is_banned: false,
								},
								|player| database::schemas::PlayerRow {
									id: player.steam_id.as_id32(),
									name: player.name,
									is_banned: player.is_banned,
								},
							);

						database::insert::players(&[row], &pool).await?;
					}

					let server_id = row.id;

					database::insert::servers(&[row], &pool).await?;

					server_id
				};

				let created_on = DateTime::from_utc(record.created_on, Utc);

				let row = database::schemas::RecordRow {
					id: record.id,
					course_id,
					mode_id: record.mode as u8,
					player_id,
					server_id,
					time: record.time,
					teleports: record.teleports,
					created_on,
				};

				database::insert::records(&[row], &pool).await?;
				info!("Inserted record #{current_id}. ({created_on})");
			}
		}
		ExecutionMode::Json { path } => {
			let json_input = tokio::fs::read_to_string(path).await?;
			let records: Vec<global_api::records::id::Response> =
				serde_json::from_str(&json_input)?;

			for record in records {
				let course_id = database::select::get_course_by_map(
					MapIdentifier::ID(record.map_id as u16),
					&pool,
				)
				.await?
				.id;

				let Ok(steam_id) = SteamID::new(record.steam_id.as_ref().unwrap_or(&record.steamid64)) else {
					warn!("Skipping record #{}, invalid SteamID: `{:?}` / `{}`", record.id, record.steam_id, record.steamid64);
					continue;
				};

				let player_id = if let Ok(player) =
					database::select::get_player(steam_id.into(), &pool).await
				{
					// Update player name if necessary.
					if let Some(player_name) = record.player_name {
						if player.name != player_name {
							info!(
								r#""{}" ({:?}) has changed their name to "{}"."#,
								player.name, record.steam_id, player_name,
							);
							sqlx::query!(
								"UPDATE players SET name = ? WHERE id = ?",
								player_name,
								player.id
							)
							.execute(&pool)
							.await?;
						}
					}

					// If a new record has a player but the player is marked as `banned` in the
					// database, that must mean the player got unbanned.
					if player.is_banned {
						info!(
							r#""{}" ({:?}) got unbanned, apparently."#,
							player.name, record.steam_id
						);
						sqlx::query!("UPDATE players SET is_banned = 0 WHERE id = ?", player.id)
							.execute(&pool)
							.await?;
					}

					player.id
				} else {
					info!(r#"Missing player "{:?}" ({:?})."#, record.player_name, record.steam_id);
					info!("Fetching...");

					let steam_id = SteamID::new(
						record
							.steam_id
							.as_ref()
							.unwrap_or(&record.steamid64),
					)?;

					let row = global_api::get_player(steam_id.into(), &gokz_client)
						.await
						.map_or(
							database::schemas::PlayerRow {
								id: steam_id.as_id32(),
								name: String::from("unknown"),
								is_banned: false,
							},
							|player| database::schemas::PlayerRow {
								id: player.steam_id.as_id32(),
								name: player.name,
								is_banned: player.is_banned,
							},
						);

					let player_id = row.id;

					database::insert::players(&[row], &pool).await?;

					player_id
				};

				let server_id = if let Ok(server) =
					database::select::get_server((record.server_id as u16).into(), &pool).await
				{
					// Update server name if necessary.
					if let Some(server_name) = record.server_name {
						if server.name != server_name {
							info!(
								r#"Server "{}" ({}) has changed its name to "{}"."#,
								server.name, server.id, server_name
							);
							sqlx::query!(
								"UPDATE servers SET name = ? WHERE id = ?",
								server_name,
								server.id
							)
							.execute(&pool)
							.await?;
						}
					}

					server.id
				} else {
					info!(r#"Missing server "{:?}" ({})."#, record.server_name, record.server_id);
					info!("Fetching...");

					let (row, owner_steamid) = global_api::get_server(
						&ServerIdentifier::ID(record.server_id as u16),
						&gokz_client,
					)
					.await
					.map(|server| {
						(
							database::schemas::ServerRow {
								id: server.id,
								name: server.name,
								owned_by: server.owner_steamid.as_id32(),
								approved_by: 0,
							},
							server.owner_steamid,
						)
					})?;

					if database::select::get_player(owner_steamid.into(), &pool)
						.await
						.is_err()
					{
						let row = global_api::get_player(owner_steamid.into(), &gokz_client)
							.await
							.map_or(
								database::schemas::PlayerRow {
									id: owner_steamid.as_id32(),
									name: String::from("unknown"),
									is_banned: false,
								},
								|player| database::schemas::PlayerRow {
									id: player.steam_id.as_id32(),
									name: player.name,
									is_banned: player.is_banned,
								},
							);

						database::insert::players(&[row], &pool).await?;
					}

					let server_id = row.id;

					database::insert::servers(&[row], &pool).await?;

					server_id
				};

				let Ok(created_on) = NaiveDateTime::parse_from_str(&record.created_on, "%Y-%m-%dT%H:%M:%S") else {
					warn!("Failed to parse `{}` as valid date.", record.created_on);
					continue;
				};
				let created_on = DateTime::from_utc(created_on, Utc);

				let Ok(mode) = record.mode.parse::<Mode>() else {
					warn!("Failed to parse `{}` as valid mode.", record.mode);
					continue;
				};

				let row = database::schemas::RecordRow {
					id: record.id as u32,
					course_id,
					mode_id: mode as u8,
					player_id,
					server_id,
					time: record.time,
					teleports: record.teleports as u32,
					created_on,
				};

				database::insert::records(&[row], &pool).await?;
			}
		}
	};

	Ok(())
}
