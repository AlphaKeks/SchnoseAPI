use {
	super::Mode,
	crate::{GlobalState, Response, ResponseBody},
	axum::{extract::State, Json},
	database::schemas::ModeRow,
	gokz_rs::prelude::Mode as GOKZMode,
	log::debug,
	std::time::Instant,
};

pub(crate) async fn get(State(GlobalState { pool }): State<GlobalState>) -> Response<Vec<Mode>> {
	let start = Instant::now();
	debug!("[modes::index::get]");

	let result = sqlx::query_as::<_, ModeRow>("SELECT * FROM modes")
		.fetch_all(&pool)
		.await?
		.into_iter()
		.filter_map(|mode_row| {
			let mode = GOKZMode::try_from(mode_row.id).ok()?;
			Some(Mode {
				id: mode_row.id,
				name: mode_row.name,
				name_short: mode.short(),
				name_long: mode.to_string(),
				created_on: mode_row.created_on.to_string(),
			})
		})
		.collect();

	debug!("> {result:#?}");

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
