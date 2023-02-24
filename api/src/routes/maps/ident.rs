use {
	super::{Course, Map, MapRow},
	crate::{Error, GlobalState, Response, ResponseBody},
	axum::{
		extract::{Path, State},
		Json,
	},
	database::schemas::account_id_to_steam_id64,
	gokz_rs::prelude::*,
	log::debug,
	sqlx::QueryBuilder,
	std::time::Instant,
};

pub(crate) async fn get(
	Path(map_ident): Path<String>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Map> {
	let start = Instant::now();
	debug!("[maps::ident::get]");
	debug!("> `map_ident`: {map_ident:#?}");
	let map_ident = map_ident.parse::<MapIdentifier>()?;
	debug!("> `map_ident`: {map_ident:#?}");

	if let MapIdentifier::Name(map_name) = &map_ident {
		if map_name.contains('&') {
			return Err(Error::Input {
				message: format!(
					"Interpreted `{map_name}` as a map name. You probably meant to use a `?` instead of the first `&`."
				),
				expected: String::from("?` instead of `&"),
			});
		}
	}

	let mut query = QueryBuilder::new(
		r#"
		SELECT
		  map.id,
		  map.name,
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
		  map.filesize,
		  mapper.name AS mapper_name,
		  map.created_by,
		  approver.name AS approver_name,
		  map.approved_by,
		  map.created_on,
		  map.updated_on
		FROM (
		  SELECT * FROM maps AS map
		  WHERE
		"#,
	);

	match map_ident {
		MapIdentifier::ID(map_id) => {
			query
				.push("map.id = ")
				.push_bind(map_id)
				.push(" LIMIT 1) AS map");
		}
		MapIdentifier::Name(map_name) => {
			query
				.push("map.name LIKE ")
				.push_bind(format!("%{map_name}%"))
				.push(" LIMIT 1) AS map");
		}
	};

	query.push(
		r#"
		JOIN `courses` AS c ON c.map_id = map.id
		JOIN players AS mapper ON mapper.id = map.created_by
		JOIN players AS approver ON approver.id = map.approved_by
		LIMIT 1
		"#,
	);

	let map_row = query
		.build_query_as::<MapRow>()
		.fetch_one(&pool)
		.await?;

	dbg!(&map_row);
	let courses = serde_json::from_str::<Vec<Course>>(&map_row.courses).map_err(|_| Error::JSON)?;

	let result = Map {
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
	};

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
