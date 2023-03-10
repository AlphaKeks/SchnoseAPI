use {
	super::{Server, ServerQuery},
	crate::{GlobalState, Response, ResponseBody},
	axum::{
		extract::{Path, State},
		Json,
	},
	database::schemas::{account_id_to_steam_id64, FancyPlayer},
	gokz_rs::prelude::SteamID,
	log::debug,
	sqlx::QueryBuilder,
	std::time::Instant,
};

pub(crate) async fn get(
	Path(server_ident): Path<String>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Server> {
	let start = Instant::now();
	debug!("[servers::ident::get]");
	debug!("> `server_ident`: {server_ident:#?}");

	let mut query = QueryBuilder::new(
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
		"#,
	);

	if let Ok(server_id) = server_ident.parse::<u16>() {
		query
			.push(" WHERE s.id = ")
			.push_bind(server_id);
	} else {
		query
			.push(" WHERE s.name LIKE ")
			.push_bind(format!("%{server_ident}%"));
	};

	let result = query
		.build_query_as::<ServerQuery>()
		.fetch_one(&pool)
		.await
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
		})?;

	debug!("> {result:#?}");

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
