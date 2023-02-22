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
	std::time::Instant,
};

pub(crate) async fn get(
	Path(map_ident): Path<String>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Map> {
	let start = Instant::now();
	debug!("[maps::ident::get]");
	debug!("> `map_ident`: {map_ident:#?}");

	let map_ident = if let Ok(map_id) = map_ident.parse::<u16>() {
		MapIdentifier::ID(map_id as i32)
	} else {
		MapIdentifier::Name(map_ident)
	};
	debug!("> `map_ident`: {map_ident:#?}");

	let filter = match map_ident {
		MapIdentifier::ID(map_id) => format!("map.id = {map_id}"),
		MapIdentifier::Name(map_name) => format!(r#"map.name LIKE "%{map_name}%""#),
	};

	let map_row = sqlx::query_as::<_, MapRow>(&format!(
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
		FROM maps AS map
		JOIN courses AS c ON c.map_id = map.id
		JOIN players AS mapper ON mapper.id = map.created_by
		JOIN players AS approver ON approver.id = map.approved_by
		WHERE {filter}
		LIMIT 1
		"#
	))
	.fetch_one(&pool)
	.await?;

	let courses = serde_json::from_str::<Vec<Course>>(&map_row.courses).map_err(|_| Error::JSON)?;

	let result = Map {
		id: map_row.id,
		name: map_row.name,
		tier: courses[0].kzt_difficulty,
		courses,
		validated: map_row.validated,
		mapper_steam_id64: account_id_to_steam_id64(map_row.created_by).to_string(),
		mapper_name: map_row.mapper_name,
		approver_steam_id64: account_id_to_steam_id64(map_row.approved_by).to_string(),
		approver_name: map_row.approver_name,
		filesize: map_row.filesize.to_string(),
		created_on: map_row.created_on.to_string(),
		updated_on: map_row.updated_on.to_string(),
	};

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
