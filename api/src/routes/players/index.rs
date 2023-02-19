use {
	crate::{
		models::{Response, ResponseBody},
		GlobalState,
	},
	axum::{
		extract::{Query, State},
		Json,
	},
	chrono::Utc,
	database::{crd::read::*, schemas::FancyPlayer},
	log::debug,
	serde::Deserialize,
};

#[derive(Debug, Deserialize)]
pub(crate) struct Params {
	pub(crate) is_banned: Option<bool>,
	pub(crate) total_records: Option<u32>,
	pub(crate) kzt_tp_records: Option<u32>,
	pub(crate) kzt_pro_records: Option<u32>,
	pub(crate) skz_tp_records: Option<u32>,
	pub(crate) skz_pro_records: Option<u32>,
	pub(crate) vnl_tp_records: Option<u32>,
	pub(crate) vnl_pro_records: Option<u32>,
	pub(crate) limit: Option<u32>,
}

pub(crate) async fn get(
	Query(params): Query<Params>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Vec<FancyPlayer>> {
	let start = Utc::now().timestamp_nanos();
	debug!("[players::get]");
	debug!("> `params`: {params:#?}");

	let mut filter = String::new();
	if let Some(bool) = params.is_banned {
		filter.push_str(&format!("AND player.is_banned = {bool} "));
	}

	if let Some(amount) = params.total_records {
		filter.push_str(&format!("AND player.total_records = {amount} "));
	}

	if let Some(amount) = params.kzt_tp_records {
		filter.push_str(&format!("AND player.kzt_tp_records = {amount} "));
	}

	if let Some(amount) = params.kzt_pro_records {
		filter.push_str(&format!("AND player.kzt_pro_records = {amount} "));
	}

	if let Some(amount) = params.skz_tp_records {
		filter.push_str(&format!("AND player.skz_tp_records = {amount} "));
	}

	if let Some(amount) = params.skz_pro_records {
		filter.push_str(&format!("AND player.skz_pro_records = {amount} "));
	}

	if let Some(amount) = params.vnl_tp_records {
		filter.push_str(&format!("AND player.vnl_tp_records = {amount} "));
	}

	if let Some(amount) = params.vnl_pro_records {
		filter.push_str(&format!("AND player.vnl_pro_records = {amount} "));
	}

	let filter = format!(
		"\n{}\nLIMIT {}",
		filter.replacen("AND", "WHERE", 1),
		params
			.limit
			.map_or(100, |limit| limit.min(500))
	);

	let players = get_players(QueryInput::Filter(filter), &pool).await?;

	Ok(Json(ResponseBody {
		result: players,
		took: (Utc::now().timestamp_nanos() - start) as f64 / 1_000_000f64,
	}))
}
