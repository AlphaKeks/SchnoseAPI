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
	color_eyre::eyre::eyre,
	database::{
		crd::read::*,
		schemas::{steam_id64_to_account_id, steam_id_to_account_id, FancyMap},
	},
	gokz_rs::prelude::*,
	log::debug,
	serde::Deserialize,
};

#[derive(Debug, Deserialize)]
pub(crate) struct Params {
	pub(crate) tier: Option<u8>,
	pub(crate) courses: Option<u8>,
	pub(crate) validated: Option<bool>,
	pub(crate) created_by: Option<String>,
	pub(crate) approved_by: Option<String>,
	pub(crate) limit: Option<u32>,
}

pub(crate) async fn get(
	Query(params): Query<Params>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Vec<FancyMap>> {
	let start = Utc::now().timestamp_nanos();
	debug!("[maps::get]");
	debug!("> `params`: {params:#?}");

	let mut filter = String::new();
	if let Some(tier) = params.tier {
		let tier = Tier::try_from(tier)?;
		filter.push_str(&format!("AND map.tier = {} ", tier as u8));
	}

	if let Some(courses) = params.courses {
		filter.push_str(&format!("AND map.courses = {courses} "));
	}

	if let Some(validated) = params.validated {
		filter.push_str(&format!("AND map.validated = {validated} "));
	}

	if let Some(created_by) = params.created_by {
		let ident = PlayerIdentifier::try_from(created_by)?;
		filter.push_str(&format!(
			"AND map.{} ",
			match ident {
				PlayerIdentifier::Name(name) => {
					format!(r#"creator_name LIKE "%{name}%" "#)
				}
				PlayerIdentifier::SteamID(steam_id) => {
					let account_id = steam_id_to_account_id(&steam_id.to_string())
						.ok_or(eyre!("Invalid SteamID"))?;
					format!(r#"creator_id = {account_id} "#)
				}
				PlayerIdentifier::SteamID64(steam_id64) => {
					let account_id = steam_id64_to_account_id(steam_id64)?;
					format!(r#"creator_id = {account_id} "#)
				}
			}
		));
	}

	if let Some(approved_by) = params.approved_by {
		let ident = PlayerIdentifier::try_from(approved_by)?;
		filter.push_str(&format!(
			"AND map.{} ",
			match ident {
				PlayerIdentifier::Name(name) => {
					format!(r#"approver_name LIKE "%{name}%" "#)
				}
				PlayerIdentifier::SteamID(steam_id) => {
					let account_id = steam_id_to_account_id(&steam_id.to_string())
						.ok_or(eyre!("Invalid SteamID"))?;
					format!(r#"approver_id = {account_id} "#)
				}
				PlayerIdentifier::SteamID64(steam_id64) => {
					let account_id = steam_id64_to_account_id(steam_id64)?;
					format!(r#"approver_id = {account_id} "#)
				}
			}
		));
	}

	let filter = format!(
		"\n{}\nLIMIT {}",
		filter.replacen("AND", "WHERE", 1),
		params.limit.unwrap_or(9999)
	);

	let maps = get_maps(QueryInput::Filter(filter), &pool).await?;

	Ok(Json(ResponseBody {
		result: maps,
		took: (Utc::now().timestamp_nanos() - start) as f64 / 1_000_000f64,
	}))
}
