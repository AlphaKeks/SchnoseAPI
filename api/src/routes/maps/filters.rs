use {
	super::{Filter, FiltersRow},
	crate::{Error, GlobalState, Response, ResponseBody},
	axum::{
		extract::{Query, State},
		Json,
	},
	database::crd::read::*,
	gokz_rs::prelude::*,
	log::debug,
	serde::Deserialize,
	sqlx::QueryBuilder,
	std::time::Instant,
};

#[derive(Debug, Deserialize)]
pub(crate) struct Params {
	name: Option<String>,
	mode: Option<String>,
	tier: Option<u8>,
	stage: Option<u8>,
	validated: Option<bool>,
	created_by: Option<String>,
	approved_by: Option<String>,
}

pub(crate) async fn get(
	Query(params): Query<Params>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Vec<Filter>> {
	let start = Instant::now();
	debug!("[maps::get]");
	debug!("> `params`: {params:#?}");

	let mut query = QueryBuilder::new(
		r#"
		SELECT
		  JSON_ARRAYAGG(
		    JSON_OBJECT(
		      "course_id", c.id,
		      "map_id", map.id,
		      "map_name", map.name,
		      "stage", c.stage,
		      "kzt", c.kzt,
		      "skz", c.skz,
		      "vnl", c.vnl
		    )
		  ) AS courses
		FROM (
		  SELECT map.* FROM maps AS map
		  JOIN courses AS c ON c.map_id = map.id
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

	if let Some(mode_ident) = params.mode {
		let mode = mode_ident.parse::<Mode>()?;
		query
			.push(" WHERE ")
			.push(&format!(" c.{} = 1", mode.short().to_lowercase()));
		multiple_filters = true;
	}

	if let Some(stage) = params.stage {
		query
			.push(if multiple_filters { " AND " } else { " WHERE " })
			.push(" c.stage = ")
			.push_bind(stage);
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
			.push(" map.approved_by = ")
			.push_bind(player.id);
	}

	query.push(
		r#"
			  GROUP BY c.map_id
			) AS map
			JOIN courses AS c ON c.map_id = map.id
			"#,
	);

	if let Some(tier) = params.tier {
		let tier = Tier::try_from(tier)?;
		query
			.push(" AND c.kzt_difficulty = ")
			.push_bind(tier as u8);
	}

	query.push(
		r#"
		ORDER BY map.name, c.stage
		"#,
	);

	let result = query
		.build_query_as::<FiltersRow>()
		.fetch_one(&pool)
		.await?;

	let result = serde_json::from_str::<Vec<Filter>>(&result.courses).map_err(|_| Error::JSON)?;

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
