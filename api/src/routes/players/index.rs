use {
	crate::{GlobalState, Response, ResponseBody},
	axum::{
		extract::{Query, State},
		Json,
	},
	database::schemas::PlayerRow,
	log::debug,
	serde::Deserialize,
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

	let mut filter = None;
	if let Some(is_banned) = params.is_banned {
		filter = Some(format!("WHERE p.is_banned = {}", is_banned as u8));
	}

	let result = sqlx::query_as::<_, PlayerRow>(&format!(
		r#"
		SELECT * FROM players AS p
		{}
		ORDER BY p.id DESC
		LIMIT {}
		OFFSET {}
		"#,
		filter.unwrap_or_default(),
		params
			.limit
			.map_or(100, |limit| limit.min(500)),
		params.offset.unwrap_or(0)
	))
	.fetch_all(&pool)
	.await?;

	debug!("> {result:#?}");

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
