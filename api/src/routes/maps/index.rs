use {
	super::{Course, Map, MapRow},
	crate::{GlobalState, Response, ResponseBody},
	axum::{
		extract::{Query, State},
		Json,
	},
	database::{crd::read::*, schemas::account_id_to_steam_id64},
	gokz_rs::prelude::*,
	log::debug,
	serde::Deserialize,
	std::time::Instant,
};

#[derive(Debug, Deserialize)]
pub(crate) struct Params {
	tier: Option<u8>,
	courses: Option<u8>,
	validated: Option<bool>,
	created_by: Option<String>,
	approved_by: Option<String>,
	limit: Option<u32>,
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

	let result = sqlx::query_as::<_, MapRow>(&format!(
		r#"
		SELECT
		  m.id AS id,
		  m.name AS name,
		  c.kzt_difficulty AS tier,
		  JSON_ARRAYAGG(
		    JSON_OBJECT(
		      "id", c.id,
		      "stage", c.stage,
		      "kzt", c.kzt,
		      "kzt_difficulty", c.kzt_difficulty,
		      "skz", c.skz,
		      "skz_difficulty", c.skz_difficulty,
		      "vnl", c.vnl,
		      "vnl_difficulty", c.vnl_difficulty
		    )
		  ) AS courses,
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
	.filter_map(|map_row| {
		let courses = serde_json::from_str::<Vec<Course>>(&map_row.courses).ok()?;
		Some(Map {
			id: map_row.id,
			name: map_row.name,
			tier: courses[0].kzt_difficulty,
			courses,
			validated: map_row.validated,
			mapper_name: map_row.mapper_name,
			mapper_steam_id64: account_id_to_steam_id64(map_row.created_by).to_string(),
			approver_name: map_row.approver_name,
			approver_steam_id64: account_id_to_steam_id64(map_row.approved_by).to_string(),
			filesize: map_row.filesize.to_string(),
			created_on: map_row.created_on.to_string(),
			updated_on: map_row.updated_on.to_string(),
		})
	})
	.collect();

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
