use {
	crate::GlobalState,
	axum::{
		extract::{Json, State},
		http::{HeaderMap, StatusCode},
	},
	gokz_rs::prelude::{Mode, SteamID},
	log::{debug, error},
	serde::{Deserialize, Serialize},
	sqlx::FromRow,
};

/// Information payload sent by the GSI desktop client for SchnoseBot (Twitch).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
	pub player_name: String,
	pub steam_id: SteamID,
	pub mode: Option<String>,
	pub map: Option<MapInfo>,
}

/// Information about the current map being played.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapInfo {
	pub name: String,
	pub tier: Option<u8>,
}

pub(crate) async fn post(
	headers: HeaderMap,
	State(GlobalState { pool }): State<GlobalState>,
	Json(info): Json<Info>,
) -> StatusCode {
	debug!("Headers: {headers:?}");
	debug!("Body: {info:?}");

	let Some(api_key) = headers.get("x-schnose-auth-key") else {
		debug!("missing API key header");
		return StatusCode::BAD_REQUEST
	};

	let Ok(api_key) = api_key.to_str() else {
		debug!("API key is not a string");
		return StatusCode::BAD_REQUEST
	};

	let api_key = ApiKey(api_key.to_owned());

	let Ok(api_keys) = sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys").fetch_all(&pool).await else {
		debug!("failed to fetch API keys");
		return StatusCode::INTERNAL_SERVER_ERROR
	};

	if !api_keys.contains(&api_key) {
		debug!("invalid API key ({api_key:?})");
		debug!("api keys: {api_keys:?}");
		return StatusCode::UNAUTHORIZED;
	}

	let (map_name, map_tier) = match &info.map {
		Some(map) => (Some(map.name.clone()), Some(map.tier)),
		None => (None, None),
	};

	debug!("Updating database with: {info:#?}");

	if let Err(why) = sqlx::query!(
		r#"
		UPDATE streamers
		SET
		  player_name = ?,
		  steam_id = ?,
		  mode = ?,
		  map_name = ?,
		  map_tier = ?
		WHERE
		  api_key = ?
		"#,
		info.player_name,
		info.steam_id.to_string(),
		info.mode.and_then(|mode| mode
			.parse::<Mode>()
			.ok()
			.map(|mode| mode.api())),
		map_name,
		map_tier,
		api_key.0
	)
	.execute(&pool)
	.await
	{
		error!("Failed updating database: {why:?}");
		return StatusCode::INTERNAL_SERVER_ERROR;
	}

	StatusCode::OK
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, FromRow)]
struct ApiKey(String);
