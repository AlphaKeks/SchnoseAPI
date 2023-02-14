use {
	crate::schemas::{
		self, account_id_to_steam_id64, steam_id64_to_account_id, steam_id_to_account_id,
	},
	color_eyre::{eyre::eyre, Result as Eyre},
	gokz_rs::prelude::*,
	log::{debug, error, info, trace, warn},
	sqlx::{MySql, Pool},
};

pub async fn get_modes(pool: &Pool<MySql>) -> Eyre<Vec<schemas::Mode>> {
	let query = String::from(r#"SELECT * FROM modes"#);
	debug!("[get_modes] Query: {query}");

	Ok(sqlx::query_as::<_, schemas::raw::ModeRow>(&query)
		.fetch_all(pool)
		.await?
		.into_iter()
		.map(|row| {
			debug!("Parsing row {row:?}");
			let mode = Mode::from(&row);
			schemas::Mode {
				id: mode as u8,
				name: mode.api(),
				name_short: mode.short(),
				name_long: mode.to_string(),
				created_on: row.created_on,
			}
		})
		.collect())
}

pub async fn get_mode(mode: Mode, pool: &Pool<MySql>) -> Eyre<schemas::Mode> {
	let query = format!(r#"SELECT * FROM modes WHERE id = {}"#, mode as u8);
	debug!("[get_mode] Query: {query}");

	Ok(sqlx::query_as::<_, schemas::raw::ModeRow>(&query)
		.fetch_one(pool)
		.await
		.map(|row| {
			debug!("Parsing row {row:?}");
			let mode = Mode::from(&row);
			schemas::Mode {
				id: mode as u8,
				name: mode.api(),
				name_short: mode.short(),
				name_long: mode.to_string(),
				created_on: row.created_on,
			}
		})?)
}

pub async fn get_players(
	limit: Option<u32>,
	custom_query: Option<String>,
	pool: &Pool<MySql>,
) -> Eyre<Vec<schemas::FancyPlayer>> {
	let query = match custom_query {
		Some(query) => query,
		None => {
			let limit = match limit {
				Some(limit) => format!("LIMIT {limit}"),
				None => String::new(),
			};
			format!(
				r#"
				SELECT
				  p.id AS account_id,
				  p.name AS name,
				  p.is_banned AS is_banned,
				  COUNT(*) AS total_records,
				  SUM(r.mode_id = 200 AND r.teleports = 0) AS kzt_tp_records,
				  SUM(r.mode_id = 200 AND r.teleports > 0) AS kzt_pro_records,
				  SUM(r.mode_id = 201 AND r.teleports = 0) AS skz_tp_records,
				  SUM(r.mode_id = 201 AND r.teleports > 0) AS skz_pro_records,
				  SUM(r.mode_id = 202 AND r.teleports = 0) AS vnl_tp_records,
				  SUM(r.mode_id = 202 AND r.teleports > 0) AS vnl_pro_records
				FROM players as p
				JOIN records AS r ON p.id = r.player_id
				GROUP BY p.id
				ORDER BY p.id
				{limit}
				"#,
			)
		}
	};
	debug!("[get_players] Query: {query}");

	let players = sqlx::query_as::<_, schemas::Player>(&query)
		.fetch_all(pool)
		.await?
		.into_iter()
		.map(|row| {
			debug!("Parsing row {row:?}");
			let steam_id64 = account_id_to_steam_id64(row.account_id);
			let steam_id = SteamID::from(steam_id64);
			schemas::FancyPlayer {
				account_id: row.account_id,
				steam_id,
				steam_id64,
				name: row.name,
				is_banned: row.is_banned,
				total_records: row.total_records,
				kzt_tp_records: row.kzt_tp_records,
				kzt_pro_records: row.kzt_pro_records,
				skz_tp_records: row.skz_tp_records,
				skz_pro_records: row.skz_pro_records,
				vnl_tp_records: row.vnl_tp_records,
				vnl_pro_records: row.vnl_pro_records,
			}
		})
		.collect::<Vec<_>>();

	if players.is_empty() {
		Err(eyre!("NO PLAYERS FOUND"))
	} else {
		Ok(players)
	}
}

pub async fn get_player(
	player: &PlayerIdentifier,
	pool: &Pool<MySql>,
) -> Eyre<schemas::FancyPlayer> {
	let filter = match player {
		PlayerIdentifier::Name(name) => format!(r#"WHERE p.name LIKE "%{name}%""#),
		PlayerIdentifier::SteamID(steam_id) => {
			let account_id =
				steam_id_to_account_id(&steam_id.to_string()).ok_or(eyre!("Invalid SteamID"))?;
			format!(r#"WHERE p.id = {account_id}"#)
		}
		PlayerIdentifier::SteamID64(steam_id64) => {
			let account_id = steam_id64_to_account_id(*steam_id64);
			format!(r#"WHERE p.id = {account_id}"#)
		}
	};

	let query = format!(
		r#"
		SELECT
		  p.id AS account_id,
		  p.name AS name,
		  p.is_banned AS is_banned,
		  COUNT(*) AS total_records,
		  SUM(r.mode_id = 200 AND r.teleports = 0) AS kzt_tp_records,
		  SUM(r.mode_id = 200 AND r.teleports > 0) AS kzt_pro_records,
		  SUM(r.mode_id = 201 AND r.teleports = 0) AS skz_tp_records,
		  SUM(r.mode_id = 201 AND r.teleports > 0) AS skz_pro_records,
		  SUM(r.mode_id = 202 AND r.teleports = 0) AS vnl_tp_records,
		  SUM(r.mode_id = 202 AND r.teleports > 0) AS vnl_pro_records
		FROM players as p
		JOIN records AS r ON p.id = r.player_id
		{filter}
		GROUP BY p.id
		ORDER BY p.id
		LIMIT 1
		"#,
	);
	debug!("[get_players] Query: {query}");

	Ok(get_players(None, Some(query), pool)
		.await?
		.remove(0))
}

pub async fn get_servers(
	limit: Option<u32>,
	custom_query: Option<String>,
	pool: &Pool<MySql>,
) -> Eyre<Vec<schemas::FancyServer>> {
	let query = match custom_query {
		Some(query) => query,
		None => {
			let limit = match limit {
				Some(limit) => format!("LIMIT {limit}"),
				None => String::new(),
			};
			format!(r#"SELECT * FROM servers {limit}"#,)
		}
	};
	debug!("[get_servers] Query: {query}");

	let servers = sqlx::query_as::<_, schemas::Server>(&query)
		.fetch_all(pool)
		.await?
		.into_iter()
		.map(|row| {
			debug!("Parsing row {row:?}");
			schemas::FancyServer {
				id: row.id,
				name: row.name,
				owned_by: schemas::raw::PlayerRow {
					id: row.owner_id,
					name: row.owner_name,
					is_banned: row.owner_is_banned,
				},
				approved_by: schemas::raw::PlayerRow {
					id: row.approved_by_id,
					name: row.approved_by_name,
					is_banned: row.approved_by_is_banned,
				},
			}
		})
		.collect::<Vec<_>>();

	if servers.is_empty() {
		Err(eyre!("NO SERVERS FOUND"))
	} else {
		Ok(servers)
	}
}

pub async fn get_server_by_id(server_id: u16, pool: &Pool<MySql>) -> Eyre<schemas::FancyServer> {
	let query = format!(
		r#"
		SELECT * FROM servers
		WHERE id = {server_id}
		"#
	);
	debug!("[get_server_by_id] Query: {query}");

	Ok(get_servers(None, Some(query), pool)
		.await?
		.remove(0))
}

pub async fn get_server_by_name(
	server_name: &str,
	pool: &Pool<MySql>,
) -> Eyre<schemas::FancyServer> {
	let query = format!(
		r#"
		SELECT * FROM servers
		WHERE name LIKE "%{server_name}%"
		"#
	);
	debug!("[get_server_by_name] Query: {query}");

	Ok(get_servers(None, Some(query), pool)
		.await?
		.remove(0))
}

pub async fn get_server_by_owner(
	owner: &PlayerIdentifier,
	pool: &Pool<MySql>,
) -> Eyre<schemas::FancyServer> {
	let filter = {
		let owner_id = match owner {
			player @ PlayerIdentifier::Name(_) => {
				let player = get_player(player, pool).await?;
				player.account_id
			}
			PlayerIdentifier::SteamID(steam_id) => {
				steam_id_to_account_id(&steam_id.to_string()).ok_or(eyre!("BAD STEAMID"))?
			}
			PlayerIdentifier::SteamID64(steam_id64) => steam_id64_to_account_id(*steam_id64),
		};

		format!("WHERE owned_by = {owner_id}")
	};

	let query = format!(
		r#"
		SELECT * FROM servers
		{filter}
		"#
	);
	debug!("[get_server_by_owner] Query: {query}");

	Ok(get_servers(None, Some(query), pool)
		.await?
		.remove(0))
}

pub async fn get_server_by_approver(
	approver: &PlayerIdentifier,
	pool: &Pool<MySql>,
) -> Eyre<schemas::FancyServer> {
	let filter = {
		let approved_by = match approver {
			player @ PlayerIdentifier::Name(_) => {
				let player = get_player(player, pool).await?;
				player.account_id
			}
			PlayerIdentifier::SteamID(steam_id) => {
				steam_id_to_account_id(&steam_id.to_string()).ok_or(eyre!("BAD STEAMID"))?
			}
			PlayerIdentifier::SteamID64(steam_id64) => steam_id64_to_account_id(*steam_id64),
		};

		format!("WHERE approved_by = {approved_by}")
	};

	let query = format!(
		r#"
		SELECT * FROM servers
		{filter}
		"#
	);
	debug!("[get_server_by_approver] Query: {query}");

	Ok(get_servers(None, Some(query), pool)
		.await?
		.remove(0))
}

pub async fn get_maps(
	validated: Option<bool>,
	limit: Option<u32>,
	custom_query: Option<String>,
	pool: &Pool<MySql>,
) -> Eyre<Vec<schemas::FancyMap>> {
	let query = match custom_query {
		Some(query) => query,
		None => {
			let filter = match (validated, limit) {
				(Some(validated), Some(limit)) => {
					format!("WHERE m.validated = {validated} LIMIT {limit}")
				}
				(Some(validated), None) => format!("WHERE m.validated = {validated}"),
				(None, Some(limit)) => format!("LIMIT {limit}"),
				(None, None) => String::new(),
			};
			format!(
				r#"
				SELECT
				  m.id AS id,
				  m.name AS name,
				  co.kzt_difficulty AS tier,
				  m.courses AS courses,
				  m.validated AS validated,
				  m.filesize AS filesize,
				  cr.id AS creator_id,
				  cr.name AS creator_name,
				  cr.is_banned AS creator_is_banned,
				  ap.id AS approver_id,
				  ap.name AS approver_name,
				  ap.is_banned AS approver_is_banned,
				  m.created_on AS created_on,
				  m.updated_on AS updated_on
				FROM maps AS m
				JOIN courses AS co ON co.map_id = m.id
				JOIN players AS cr ON cr.id = m.created_by
				JOIN players AS ap ON ap.id = m.approved_by
				{filter}
				"#
			)
		}
	};
	debug!("[get_maps] Query: {query}");

	let maps = sqlx::query_as::<_, schemas::Map>(&query)
		.fetch_all(pool)
		.await?
		.into_iter()
		.filter_map(|row| {
			debug!("Parsing row {row:?}");
			Some(schemas::FancyMap {
				id: row.id,
				name: row.name,
				tier: Tier::try_from(row.tier).ok()?,
				courses: row.courses,
				validated: row.validated,
				filesize: row.filesize,
				created_by: schemas::raw::PlayerRow {
					id: row.creator_id,
					name: row.creator_name,
					is_banned: row.creator_is_banned,
				},
				approved_by: schemas::raw::PlayerRow {
					id: row.approver_id,
					name: row.approver_name,
					is_banned: row.approver_is_banned,
				},
				created_on: row.created_on,
				updated_on: row.updated_on,
			})
		})
		.collect::<Vec<_>>();

	if maps.is_empty() {
		Err(eyre!("NO MAPS FOUND"))
	} else {
		Ok(maps)
	}
}

pub async fn get_map(map: &MapIdentifier, pool: &Pool<MySql>) -> Eyre<schemas::FancyMap> {
	let query = {
		let filter = match map {
			MapIdentifier::ID(map_id) => format!("WHERE m.id = {map_id}"),
			MapIdentifier::Name(map_name) => format!(r#"WHERE m.name LIKE "%{map_name}%""#),
		};

		format!(
			r#"
			SELECT
			  m.id AS id,
			  m.name AS name,
			  co.kzt_difficulty AS tier,
			  m.courses AS courses,
			  m.validated AS validated,
			  m.filesize AS filesize,
			  cr.id AS creator_id,
			  cr.name AS creator_name,
			  cr.is_banned AS creator_is_banned,
			  ap.id AS approver_id,
			  ap.name AS approver_name,
			  ap.is_banned AS approver_is_banned,
			  m.created_on AS created_on,
			  m.updated_on AS updated_on
			FROM maps AS m
			JOIN courses AS co ON co.map_id = m.id
			JOIN players AS cr ON cr.id = m.created_by
			JOIN players AS ap ON ap.id = m.approved_by
			{filter}
			"#
		)
	};
	debug!("[get_map] Query: {query}");

	Ok(get_maps(None, None, Some(query), pool)
		.await?
		.remove(0))
}
