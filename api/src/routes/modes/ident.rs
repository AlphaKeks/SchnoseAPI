use {
	super::Mode,
	crate::{
		models::{Response, ResponseBody},
		GlobalState,
	},
	axum::{
		extract::{Path, State},
		Json,
	},
	database::schemas::ModeRow,
	gokz_rs::prelude::Mode as GOKZMode,
	log::debug,
	std::time::Instant,
};

pub(crate) async fn get(
	Path(mode_ident): Path<String>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Mode> {
	let start = Instant::now();
	debug!("[modes::ident::get]");
	debug!("> `mode_ident`: {mode_ident:#?}");

	let mode = mode_ident.parse::<GOKZMode>()?;
	debug!("> `mode`: {mode:#?}");

	let result = sqlx::query_as::<_, ModeRow>(&format!(
		r#"
		SELECT * FROM modes
		WHERE id = {}
		"#,
		mode as u8
	))
	.fetch_one(&pool)
	.await
	.map(|mode_row| Mode {
		id: mode_row.id,
		name: mode_row.name,
		name_short: mode.short(),
		name_long: mode.to_string(),
		created_on: mode_row.created_on.to_string(),
	})?;

	debug!("> {result:#?}");

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
