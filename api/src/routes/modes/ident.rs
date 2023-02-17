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
	database::{crd::read::*, schemas::Mode},
	gokz_rs::prelude::Mode as GOKZMode,
	log::debug,
};

pub(crate) async fn get(
	Path(mode_ident): Path<String>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Mode> {
	let start = Utc::now().timestamp_nanos();
	debug!("[modes::ident::get]");
	debug!("> `mode_ident`: {mode_ident:#?}");

	let mode = mode_ident.parse::<GOKZMode>()?;
	debug!("> `mode`: {mode:#?}");

	let mode = get_mode(mode, &pool).await?;

	Ok(Json(ResponseBody {
		result: mode,
		took: (Utc::now().timestamp_nanos() - start) as f64 / 1_000_000f64,
	}))
}
