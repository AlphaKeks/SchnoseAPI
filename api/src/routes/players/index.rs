use {
	crate::{GlobalState, Response, ResponseBody},
	axum::{
		extract::{Query, State},
		Json,
	},
	database::schemas::PlayerRow,
	log::debug,
	serde::Deserialize,
	sqlx::QueryBuilder,
	std::time::Instant,
};

#[derive(Debug, Deserialize)]
pub(crate) struct Params {
	is_banned: Option<bool>,
	limit: Option<u32>,
	offset: Option<i32>,
}

pub(crate) async fn get(
	Query(params): Query<Params>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Vec<PlayerRow>> {
	let start = Instant::now();
	debug!("[players::get]");
	debug!("> `params`: {params:#?}");

	let mut query = QueryBuilder::new(
		r#"
		SELECT * FROM players AS p
		"#,
	);

	if let Some(is_banned) = params.is_banned {
		query
			.push("WHERE p.is_banned = ")
			.push_bind(is_banned);
	}

	query
		.push(" ORDER BY p.id DESC ")
		.push(" LIMIT ")
		.push_bind(
			params
				.limit
				.map_or(100, |limit| limit.min(500)),
		)
		.push(" OFFSET ")
		.push_bind(params.offset.unwrap_or(0));

	let result = query
		.build_query_as::<PlayerRow>()
		.fetch_all(&pool)
		.await?;

	debug!("> {result:#?}");

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
