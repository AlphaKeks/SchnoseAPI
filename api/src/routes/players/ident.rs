use {
	crate::{
		models::{Response, ResponseBody},
		GlobalState,
	},
	axum::{
		extract::{Path, State},
		Json,
	},
	chrono::Utc,
	color_eyre::eyre::eyre,
	database::{
		crd::read::*,
		schemas::{steam_id64_to_account_id, steam_id_to_account_id, FancyPlayer},
	},
	gokz_rs::prelude::*,
	log::debug,
};

pub(crate) async fn get(
	Path(player_ident): Path<String>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<FancyPlayer> {
	let start = Utc::now().timestamp_nanos();
	debug!("[players::ident::get]");
	debug!("> `player_ident`: {player_ident:#?}");

	let player_ident = player_ident.parse::<PlayerIdentifier>()?;
	debug!("> `player_ident`: {player_ident:#?}");

	let filter = format!(
		r#"
		WHERE player.{}
		LIMIT 1
		"#,
		match player_ident {
			PlayerIdentifier::Name(name) => format!(r#"name LIKE "%{name}%""#),
			PlayerIdentifier::SteamID(steam_id) => {
				let account_id = steam_id_to_account_id(&steam_id.to_string())
					.ok_or(eyre!("Invalid SteamID"))?;
				format!(r#"account_id = {account_id}"#)
			}
			PlayerIdentifier::SteamID64(steam_id64) => {
				let account_id = steam_id64_to_account_id(steam_id64)?;
				format!(r#"account_id = {account_id}"#)
			}
		}
	);

	let player = get_players(QueryInput::Filter(filter), &pool)
		.await?
		.remove(0);

	Ok(Json(ResponseBody {
		result: player,
		took: (Utc::now().timestamp_nanos() - start) as f64 / 1_000_000f64,
	}))
}
