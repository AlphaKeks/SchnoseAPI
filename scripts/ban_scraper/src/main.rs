#![deny(clippy::perf)]
#![warn(clippy::suspicious, clippy::style)]

use {
	chrono::NaiveDateTime,
	clap::Parser,
	color_eyre::Result as Eyre,
	database::schemas::steam_id64_to_account_id,
	gokz_rs::bans::Ban,
	log::info,
	serde::Deserialize,
	sqlx::mysql::MySqlPoolOptions,
	std::{collections::HashSet, path::PathBuf, time::Duration},
};

#[derive(Debug, Parser)]
struct Args {
	/// Config file containing a MySQL connection string
	#[arg(short, long)]
	#[clap(default_value = "./config.toml")]
	config_file: PathBuf,

	/// Print debug information
	#[arg(long)]
	#[clap(default_value = "false")]
	debug: bool,
}

#[derive(Debug, Deserialize)]
struct Config {
	mysql_url: String,
}

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;
	let args = Args::parse();
	let config_file = std::fs::read_to_string(args.config_file)?;
	let config: Config = toml::from_str(&config_file)?;

	std::env::set_var("RUST_LOG", if args.debug { "DEBUG" } else { "ban_scraper=INFO" });
	env_logger::init();

	let pool = MySqlPoolOptions::new()
		.connect(&config.mysql_url)
		.await?;

	let client = gokz_rs::Client::new();

	let mut old_bans = HashSet::new();
	loop {
		let new_bans = client
			.get("https://kztimerglobal.com/api/v2/bans?limit=100000")
			.send()
			.await?
			.json::<Vec<Ban>>()
			.await?
			.into_iter()
			.filter_map(|ban| {
				let expiration_date =
					NaiveDateTime::parse_from_str(&ban.expires_on, "%Y-%m-%dT%H:%M:%S").ok()?;
				let creation_date =
					NaiveDateTime::parse_from_str(&ban.created_on, "%Y-%m-%dT%H:%M:%S").ok()?;
				let is_banned = expiration_date <= creation_date;

				Some((ban.steamid64, is_banned))
			})
			.collect::<HashSet<_>>();

		let changes = new_bans.difference(&old_bans);

		for (steam_id64, is_banned) in changes {
			let Ok(steam_id64) = steam_id64.parse::<u64>() else {
				continue;
			};
			let Ok(player_id) = steam_id64_to_account_id(steam_id64) else {
				continue;
			};
			let is_banned = *is_banned as u8;

			_ = sqlx::query(&format!(
				r#"
				UPDATE IGNORE players
				SET is_banned = {is_banned}
				WHERE id = {player_id}
				"#
			))
			.execute(&pool)
			.await;

			info!("Updated ban for `{player_id}` to `{is_banned}`.");
		}

		old_bans = new_bans;

		std::thread::sleep(Duration::from_secs(120));
	}
}
