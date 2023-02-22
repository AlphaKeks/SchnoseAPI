use {
	crate::{GlobalState, Response, ResponseBody},
	axum::{
		extract::{Path, State},
		Json,
	},
	database::{crd::read::get_player, schemas::account_id_to_steam_id64},
	gokz_rs::prelude::*,
	log::debug,
	serde::Serialize,
	sqlx::{types::Decimal, FromRow},
	std::time::Instant,
};

#[derive(Debug, FromRow)]
struct DBPlayer {
	id: u32,
	name: String,
	is_banned: bool,
	total: i64,
	kzt_tp: Decimal,
	kzt_pro: Decimal,
	skz_tp: Decimal,
	skz_pro: Decimal,
	vnl_tp: Decimal,
	vnl_pro: Decimal,
}

#[derive(Debug, Serialize)]
pub struct Player {
	id: u32,
	name: String,
	steam_id: String,
	steam_id64: String,
	is_banned: bool,
	records: RecordSummary,
}

#[derive(Debug, Serialize)]
pub struct RecordSummary {
	total: u32,
	kzt: RecordCount,
	skz: RecordCount,
	vnl: RecordCount,
}

#[derive(Debug, Serialize)]
pub struct RecordCount {
	tp: u32,
	pro: u32,
}

pub(crate) async fn get(
	Path(player_ident): Path<String>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Player> {
	let start = Instant::now();
	debug!("[players::ident::get]");
	debug!("> `player_ident`: {player_ident:#?}");

	let player_ident = player_ident.parse::<PlayerIdentifier>()?;
	debug!("> `player_ident`: {player_ident:#?}");

	let player = get_player(player_ident, &pool).await?;

	let result = sqlx::query_as::<_, DBPlayer>(&format!(
		r#"
		SELECT
		  p.id                                     AS id,
		  p.name                                   AS name,
		  p.is_banned                              AS is_banned,
		  COUNT(*)                                 AS total,
		  SUM(r.mode_id = 200 AND r.teleports > 0) AS kzt_tp,
		  SUM(r.mode_id = 200 AND r.teleports = 0) AS kzt_pro,
		  SUM(r.mode_id = 201 AND r.teleports > 0) AS skz_tp,
		  SUM(r.mode_id = 201 AND r.teleports = 0) AS skz_pro,
		  SUM(r.mode_id = 202 AND r.teleports > 0) AS vnl_tp,
		  SUM(r.mode_id = 202 AND r.teleports = 0) AS vnl_pro
		FROM players AS p
		JOIN records AS r ON r.player_id = p.id
		WHERE p.id = {}
		GROUP BY p.id
		LIMIT 1
		"#,
		player.id
	))
	.fetch_one(&pool)
	.await
	.map(|db_player| {
		let steam_id64 = account_id_to_steam_id64(db_player.id);
		let steam_id = SteamID::from(steam_id64);
		Player {
			id: db_player.id,
			name: db_player.name,
			steam_id: steam_id.to_string(),
			steam_id64: steam_id64.to_string(),
			is_banned: db_player.is_banned,
			records: RecordSummary {
				total: db_player.total as u32,
				kzt: RecordCount {
					tp: db_player.kzt_tp.try_into().unwrap(),
					pro: db_player.kzt_pro.try_into().unwrap(),
				},
				skz: RecordCount {
					tp: db_player.skz_tp.try_into().unwrap(),
					pro: db_player.skz_pro.try_into().unwrap(),
				},
				vnl: RecordCount {
					tp: db_player.vnl_tp.try_into().unwrap(),
					pro: db_player.vnl_pro.try_into().unwrap(),
				},
			},
		}
	})?;

	debug!("> {result:#?}");

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
