use color_eyre::{eyre::eyre, Result as Eyre};
use log::info;
use serde::{Deserialize, Serialize};

pub async fn get_player(steam_id64: u64, key: &str, client: &gokz_rs::Client) -> Eyre<Player> {
	info!("FETCHING PLAYER `{steam_id64}`");
	let url = format!(
		"http://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={key}&steamids={steam_id64}"
	);
	let res = client.get(&url).send().await?;
	let mut json = res.json::<OuterResponse>().await?;
	if json.response.players.is_empty() {
		return Err(eyre!("NO PLAYERS"));
	}
	Ok(json.response.players.remove(0))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OuterResponse {
	response: InnerResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InnerResponse {
	players: Vec<Player>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
	pub steamid: Option<String>,
	pub communityvisibilitystate: Option<i32>,
	pub profilestate: Option<i32>,
	pub personaname: Option<String>,
	pub commentpermission: Option<i32>,
	pub profileurl: Option<String>,
	pub avatar: Option<String>,
	pub avatarmedium: Option<String>,
	pub avatarfull: Option<String>,
	pub avatarhash: Option<String>,
	pub lastlogoff: Option<u32>,
	pub personastate: Option<i32>,
	pub realname: Option<String>,
	pub primaryclanid: Option<String>,
	pub timecreated: Option<u32>,
	pub personastateflags: Option<i32>,
	pub loccountrycode: Option<String>,
}
