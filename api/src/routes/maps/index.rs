use {
	crate::{
		models::{Response, ResponseBody},
		GlobalState,
	},
	axum::{
		extract::{Query, State},
		Json,
	},
	database::{crd::read::*, schemas::account_id_to_steam_id64},
	gokz_rs::prelude::*,
	log::debug,
	serde::{Deserialize, Serialize},
	sqlx::{types::time::PrimitiveDateTime, FromRow},
	std::time::Instant,
};

#[derive(Debug, Deserialize)]
pub struct Params {
	tier: Option<u8>,
	courses: Option<u8>,
	validated: Option<bool>,
	created_by: Option<String>,
	approved_by: Option<String>,
	limit: Option<u32>,
}

#[derive(Debug, FromRow)]
struct MapQuery {
	id: u16,
	name: String,
	tier: u8,
	courses: i64,
	validated: bool,
	mapper_name: String,
	created_by: u32,
	approver_name: String,
	approved_by: u32,
	filesize: u64,
	created_on: PrimitiveDateTime,
	updated_on: PrimitiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct Map {
	id: u16,
	name: String,
	tier: u8,
	courses: u8,
	validated: bool,
	mapper_name: String,
	mapper_steam_id64: String,
	approver_name: String,
	approver_steam_id64: String,
	filesize: String,
	created_on: String,
	updated_on: String,
}

pub(crate) async fn get(
	Query(params): Query<Params>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Vec<Map>> {
	let start = Instant::now();
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
		let player = get_player(ident, &pool).await?;
		filter.push_str(&format!("AND map.created_by = {} ", player.id));
	}

	if let Some(approved_by) = params.approved_by {
		let ident = PlayerIdentifier::try_from(approved_by)?;
		let player = get_player(ident, &pool).await?;
		filter.push_str(&format!("AND map.approved_by = {} ", player.id));
	}

	let filter = format!(
		"\n{}\nLIMIT {}",
		filter.replacen("AND", "WHERE", 1),
		params
			.limit
			.map_or(1500, |limit| limit.min(1500))
	);

	let result = sqlx::query_as::<_, MapQuery>(&format!(
		r#"
		SELECT
		  m.id AS id,
		  m.name AS name,
		  c.kzt_difficulty AS tier,
		  COUNT(c.map_id) AS courses,
		  m.validated AS validated,
		  mapper.name AS mapper_name,
		  m.created_by,
		  approver.name AS approver_name,
		  m.approved_by,
		  m.filesize AS filesize,
		  m.created_on AS created_on,
		  m.updated_on AS updated_on
		FROM maps AS m
		JOIN courses AS c ON c.map_id = m.id
		JOIN players AS mapper ON mapper.id = m.created_by
		JOIN players AS approver ON approver.id = m.approved_by
		GROUP BY m.id
		ORDER BY m.created_on
		{filter}
		"#
	))
	.fetch_all(&pool)
	.await?
	.into_iter()
	.map(|map_query| Map {
		id: map_query.id,
		name: map_query.name,
		tier: map_query.tier,
		courses: map_query.courses as u8,
		validated: map_query.validated,
		mapper_name: map_query.mapper_name,
		mapper_steam_id64: account_id_to_steam_id64(map_query.created_by).to_string(),
		approver_name: map_query.approver_name,
		approver_steam_id64: account_id_to_steam_id64(map_query.approved_by).to_string(),
		filesize: map_query.filesize.to_string(),
		created_on: map_query.created_on.to_string(),
		updated_on: map_query.updated_on.to_string(),
	})
	.collect();

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
