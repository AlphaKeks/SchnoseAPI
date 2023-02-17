use {
	crate::{
		models::{Response, ResponseBody},
		GlobalState,
	},
	axum::{extract::State, Json},
	chrono::Utc,
	database::{crd::read::*, schemas::Mode},
	log::debug,
};

pub(crate) async fn get(State(GlobalState { pool }): State<GlobalState>) -> Response<Vec<Mode>> {
	let start = Utc::now().timestamp_nanos();
	debug!("[modes::index::get]");

	let modes = get_modes(&pool).await?;

	Ok(Json(ResponseBody {
		result: modes,
		took: (Utc::now().timestamp_nanos() - start) as f64 / 1_000_000f64,
	}))
}
