use {
	crate::GlobalState,
	axum::{
		extract::{Path, State},
		Json,
	},
	backend::{
		models::players::{PlayerResponse, PlayerRow},
		Response, ResponseBody,
	},
	gokz_rs::PlayerIdentifier,
	log::debug,
	tokio::time::Instant,
};

pub async fn get_by_identifier(
	Path(player_identifier): Path<PlayerIdentifier>,
	State(global_state): State<GlobalState>,
) -> Response<PlayerResponse> {
	let took = Instant::now();
	debug!("[players::get_by_identifier]");
	debug!("> `player_identifier`: {player_identifier:#?}");

	let player_id = database::select::get_player(player_identifier, &global_state.conn)
		.await?
		.id;

	let result: PlayerRow = sqlx::query_as(&format!(
		r#"
		SELECT
		  player.*,
		  COUNT(*) AS total_completions,
		  SUM(record.mode_id = 200 AND record.teleports > 0) AS kzt_tp_completions,
		  SUM(record.mode_id = 200 AND record.teleports = 0) AS kzt_pro_completions,
		  SUM(record.mode_id = 201 AND record.teleports > 0) AS skz_tp_completions,
		  SUM(record.mode_id = 201 AND record.teleports = 0) AS skz_pro_completions,
		  SUM(record.mode_id = 202 AND record.teleports > 0) AS vnl_tp_completions,
		  SUM(record.mode_id = 202 AND record.teleports = 0) AS vnl_pro_completions
		FROM players AS player
		JOIN records AS record ON record.player_id = player.id
		WHERE player.id = {player_id}
		LIMIT 1
		"#
	))
	.fetch_one(&global_state.conn)
	.await?;

	debug!("Database result: {result:#?}");

	Ok(Json(ResponseBody {
		result: result.into(),
		took: took.elapsed().as_nanos(),
	}))
}
