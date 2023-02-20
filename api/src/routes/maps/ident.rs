use {
	crate::{
		models::{Response, ResponseBody},
		GlobalState,
	},
	axum::{
		extract::{Path, State},
		Json,
	},
	database::schemas::{account_id_to_steam_id64, CourseRow},
	gokz_rs::prelude::*,
	log::debug,
	serde::Serialize,
	sqlx::{types::time::PrimitiveDateTime, FromRow},
	std::time::Instant,
};

#[derive(Debug, Clone, FromRow)]
pub struct MapRow {
	pub id: u16,
	pub name: String,
	pub courses: u8,
	pub validated: bool,
	pub filesize: u64,
	pub mapper_name: String,
	pub created_by: u32,
	pub approver_name: String,
	pub approved_by: u32,
	pub created_on: PrimitiveDateTime,
	pub updated_on: PrimitiveDateTime,
}

#[derive(Debug, Serialize)]
struct Course {
	id: u32,
	stage: u8,
	kzt: bool,
	kzt_difficulty: u8,
	skz: bool,
	skz_difficulty: u8,
	vnl: bool,
	vnl_difficulty: u8,
}

#[derive(Debug, Serialize)]
pub struct Map {
	id: u16,
	name: String,
	tier: u8,
	courses: Vec<Course>,
	validated: bool,
	pub mapper_name: String,
	pub mapper_steam_id64: String,
	pub approver_name: String,
	pub approver_steam_id64: String,
	filesize: String,
	created_on: String,
	updated_on: String,
}

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
		  map.courses,
		  map.validated,
		  map.filesize,
		  mapper.name AS mapper_name,
		  map.created_by,
		  approver.name AS approver_name,
		  map.approved_by,
		  map.created_on,
		  map.updated_on
		FROM maps AS map
		JOIN players AS mapper ON mapper.id = map.created_by
		JOIN players AS approver ON approver.id = map.approved_by
		WHERE {filter}
		LIMIT 1
		"#
	))
	.fetch_one(&pool)
	.await?;

	// TODO: Do this in the initial SQL query directly?
	let courses = sqlx::query_as::<_, CourseRow>(&format!(
		r#"
		SELECT * FROM courses
		WHERE map_id = {}
		"#,
		map_row.id,
	))
	.fetch_all(&pool)
	.await?
	.into_iter()
	.map(|course_row| Course {
		id: course_row.id,
		stage: course_row.stage,
		kzt: course_row.kzt,
		kzt_difficulty: course_row.kzt_difficulty,
		skz: course_row.skz,
		skz_difficulty: course_row.skz_difficulty,
		vnl: course_row.vnl,
		vnl_difficulty: course_row.vnl_difficulty,
	})
	.collect::<Vec<_>>();

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
