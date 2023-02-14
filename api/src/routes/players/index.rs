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
	database::{crd::read::*, schemas::FancyPlayer},
	gokz_rs::prelude::*,
	log::debug,
};

pub(crate) async fn get(
	Path(player): Path<String>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<FancyPlayer> {
	let start = Utc::now().timestamp_nanos();
	debug!("[players::get]");
	debug!("> `player`: {player:?}");

	let player = player.parse::<PlayerIdentifier>()?;

	let player = get_player(&player, &pool).await?;

	Ok(Json(ResponseBody {
		result: player,
		took: (Utc::now().timestamp_nanos() - start) as f64 / 1_000_000f64,
	}))
}
