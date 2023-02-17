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
	database::{crd::read::*, schemas::FancyServer},
	log::debug,
};

pub(crate) async fn get(
	Path(server_ident): Path<String>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<FancyServer> {
	let start = Utc::now().timestamp_nanos();
	debug!("[servers::ident::get]");
	debug!("> `server_ident`: {server_ident:#?}");

	let filter = match server_ident.parse::<u16>() {
		Ok(server_id) => {
			format!("WHERE server.id = {server_id}")
		}
		Err(_) => {
			format!(r#"WHERE server.name LIKE "%{server_ident}%""#)
		}
	};

	let server = get_servers(QueryInput::Filter(filter), &pool)
		.await?
		.remove(0);

	Ok(Json(ResponseBody {
		result: server,
		took: (Utc::now().timestamp_nanos() - start) as f64 / 1_000_000f64,
	}))
}
