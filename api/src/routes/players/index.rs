#![allow(unused)]

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
	gokz_rs::prelude::*,
	log::debug,
	serde::Deserialize,
};

#[derive(Debug, Deserialize)]
pub(crate) struct Params {}

pub(crate) async fn get(
	Query(params): Query<Params>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<FancyPlayer> {
	let start = Utc::now().timestamp_nanos();
	debug!("[players::get]");
	debug!("> `params`: {params:#?}");

	Ok(Json(ResponseBody {
		result: todo!(),
		took: (Utc::now().timestamp_nanos() - start) as f64 / 1_000_000f64,
	}))
}
