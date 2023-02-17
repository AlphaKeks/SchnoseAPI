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
	database::{crd::read::*, schemas::FancyMap},
	gokz_rs::prelude::*,
	log::debug,
};

pub(crate) async fn get(
	Path(map_ident): Path<String>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<FancyMap> {
	let start = Utc::now().timestamp_nanos();
	debug!("[maps::ident::get]");
	debug!("> `map_ident`: {map_ident:#?}");

	let map_ident = if let Ok(map_id) = map_ident.parse::<u16>() {
		MapIdentifier::ID(map_id as i32)
	} else {
		MapIdentifier::Name(map_ident)
	};
	debug!("> `map_ident`: {map_ident:#?}");

	Ok(Json(ResponseBody {
		result: get_map(&map_ident, &pool).await?,
		took: (Utc::now().timestamp_nanos() - start) as f64 / 1_000_000f64,
	}))
}
