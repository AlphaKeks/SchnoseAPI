use {
	super::{Server, ServerQuery},
	crate::{
		models::{Response, ResponseBody},
		GlobalState,
	},
	axum::{
		extract::{Query, State},
		Json,
	},
	color_eyre::eyre::eyre,
	database::schemas::{
		account_id_to_steam_id64, steam_id64_to_account_id, steam_id_to_account_id, FancyPlayer,
	},
	gokz_rs::prelude::*,
	log::debug,
	serde::Deserialize,
	std::time::Instant,
};

#[derive(Debug, Deserialize)]
pub(crate) struct Params {
	name: Option<String>,
	owned_by: Option<String>,
	approved_by: Option<String>,
	limit: Option<u32>,
}

pub(crate) async fn get(
	Query(params): Query<Params>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Vec<Server>> {
	let start = Instant::now();
	debug!("[servers::get]");
	debug!("> `params`: {params:#?}");

	let mut filter = String::new();
	if let Some(name) = params.name {
		filter.push_str(&format!(r#"AND s.name LIKE "%{name}%""#));
	}

	if let Some(owned_by) = params.owned_by {
		let ident = PlayerIdentifier::try_from(owned_by)?;
		filter.push_str(&format!(
			"AND o.{} ",
			match ident {
				PlayerIdentifier::Name(name) => {
					format!(r#"name LIKE "%{name}%" "#)
				}
				PlayerIdentifier::SteamID(steam_id) => {
					let account_id = steam_id_to_account_id(&steam_id.to_string())
						.ok_or(eyre!("Invalid SteamID"))?;
					format!(r#"id = {account_id} "#)
				}
				PlayerIdentifier::SteamID64(steam_id64) => {
					let account_id = steam_id64_to_account_id(steam_id64)?;
					format!(r#"id = {account_id} "#)
				}
			}
		));
	}

	if let Some(approved_by) = params.approved_by {
		let ident = PlayerIdentifier::try_from(approved_by)?;
		filter.push_str(&format!(
			"AND a.{} ",
			match ident {
				PlayerIdentifier::Name(name) => {
					format!(r#"name LIKE "%{name}%" "#)
				}
				PlayerIdentifier::SteamID(steam_id) => {
					let account_id = steam_id_to_account_id(&steam_id.to_string())
						.ok_or(eyre!("Invalid SteamID"))?;
					format!(r#"id = {account_id} "#)
				}
				PlayerIdentifier::SteamID64(steam_id64) => {
					let account_id = steam_id64_to_account_id(steam_id64)?;
					format!(r#"id = {account_id} "#)
				}
			}
		));
	}

	filter = filter.replacen("AND", "WHERE", 1);

	let result = sqlx::query_as::<_, ServerQuery>(&format!(
		r#"
		SELECT
		  s.id        AS id,
		  s.name      AS name,
		  o.id        AS owner_id,
		  o.name      AS owner_name,
		  o.is_banned AS owner_is_banned,
		  a.id        AS approver_id,
		  a.name      AS approver_name,
		  a.is_banned AS approver_is_banned
		FROM servers AS s
		JOIN players AS o ON o.id = s.owned_by
		JOIN players AS a ON a.id = s.approved_by
		{filter}
		LIMIT {}
		"#,
		params
			.limit
			.map_or(1500, |limit| limit.min(1500))
	))
	.fetch_all(&pool)
	.await?
	.into_iter()
	.map(|server_query| {
		let owner_steam_id64 = account_id_to_steam_id64(server_query.owner_id);
		let owner_steam_id = SteamID::from(owner_steam_id64);
		let approver_steam_id64 = account_id_to_steam_id64(server_query.approver_id);
		let approver_steam_id = SteamID::from(approver_steam_id64);

		Server {
			id: server_query.id,
			name: server_query.name,
			owned_by: FancyPlayer {
				id: server_query.owner_id,
				name: server_query.owner_name,
				steam_id: owner_steam_id.to_string(),
				steam_id64: owner_steam_id64.to_string(),
				is_banned: server_query.owner_is_banned,
			},
			approved_by: FancyPlayer {
				id: server_query.approver_id,
				name: server_query.approver_name,
				steam_id: approver_steam_id.to_string(),
				steam_id64: approver_steam_id64.to_string(),
				is_banned: server_query.approver_is_banned,
			},
		}
	})
	.collect();

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
