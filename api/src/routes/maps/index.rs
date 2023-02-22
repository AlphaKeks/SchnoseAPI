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
	sqlx::QueryBuilder,
	std::time::Instant,
};

#[derive(Debug, Deserialize)]
pub(crate) struct Params {
	name: Option<String>,
	tier: Option<u8>,
	stages: Option<u8>,
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

	let mut query = QueryBuilder::new(
		r#"
		SELECT
		  map.id,
		  map.name,
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
		  map.validated,
		  mapper.name AS mapper_name,
		  map.created_by,
		  approver.name AS approver_name,
		  map.approved_by,
		  map.filesize,
		  map.created_on,
		  map.updated_on
		FROM (
		  SELECT * FROM maps AS map
		"#,
	);

	let mut multiple_filters = false;

	if let Some(map_name) = params.name {
		query
			.push(" WHERE ")
			.push(" map.name LIKE ")
			.push_bind(format!("%{map_name}%"));
		multiple_filters = true;
	}

	if let Some(tier) = params.tier {
		let tier = Tier::try_from(tier)?;
		query
			.push(if multiple_filters { " AND " } else { " WHERE " })
			.push(" map.tier = ")
			.push_bind(tier as u8);
	}

	if let Some(courses) = params.stages {
		query
			.push(if multiple_filters { " AND " } else { " WHERE " })
			.push(" map.courses = ")
			.push_bind(courses);
	}

	if let Some(validated) = params.validated {
		query
			.push(if multiple_filters { " AND " } else { " WHERE " })
			.push(" map.validated = ")
			.push_bind(validated);
	}

	if let Some(created_by) = params.created_by {
		let ident = PlayerIdentifier::try_from(created_by)?;
		let player = get_player(ident, &pool).await?;
		query
			.push(if multiple_filters { " AND " } else { " WHERE " })
			.push(" map.created_by = ")
			.push_bind(player.id);
	}

	if let Some(approved_by) = params.approved_by {
		let ident = PlayerIdentifier::try_from(approved_by)?;
		let player = get_player(ident, &pool).await?;
		query
			.push(if multiple_filters { " AND " } else { " WHERE " })
			.push(" map.created_by = ")
			.push_bind(player.id);
	}

	query
		.push(" LIMIT ")
		.push_bind(
			params
				.limit
				.map_or(1500, |limit| limit.min(1500)),
		)
		.push(") AS map")
		.push(
			r#"
			JOIN courses AS c ON c.map_id = map.id
			JOIN players AS mapper ON mapper.id = map.created_by
			JOIN players AS approver ON approver.id = map.approved_by
			GROUP BY map.id
			ORDER BY map.created_on
			"#,
		);

	let result = query
		.build_query_as::<MapRow>()
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
