use {
	crate::{GlobalState, Response, ResponseBody},
	axum::{
		extract::{Path, State},
		Json,
	},
	database::crd::read::get_player,
	gokz_rs::prelude::*,
	log::debug,
	serde::Serialize,
	sqlx::{types::Decimal, FromRow, QueryBuilder},
	std::time::Instant,
};

#[derive(Debug, Serialize, FromRow)]
pub(crate) struct CompletionQuery {
	id: u32,
	name: String,
	is_banned: i8,
	kzt_tp: Decimal,
	kzt_pro: Decimal,
	skz_tp: Decimal,
	skz_pro: Decimal,
	vnl_tp: Decimal,
	vnl_pro: Decimal,
}

#[derive(Debug, Serialize, FromRow)]
pub(crate) struct Completion {
	id: u32,
	name: String,
	is_banned: bool,
	kzt_tp: u32,
	kzt_pro: u32,
	skz_tp: u32,
	skz_pro: u32,
	vnl_tp: u32,
	vnl_pro: u32,
}

pub(crate) async fn get(
	Path(player_ident): Path<String>,
	State(GlobalState { pool }): State<GlobalState>,
) -> Response<Completion> {
	let start = Instant::now();
	debug!("[players::completion::get]");
	debug!("> `player_ident`: {player_ident:#?}");
	let player_ident = player_ident.parse::<PlayerIdentifier>()?;
	debug!("> `player_ident`: {player_ident:#?}");

	let player = get_player(player_ident, &pool).await?;

	let mut query = QueryBuilder::new(
		r#"
		SELECT
		  p.id                 AS id,
		  p.name               AS name,
		  p.is_banned          AS is_banned,
		  SUM(r.mode_id = 200 AND r.teleports > 0) AS kzt_tp,
		  SUM(r.mode_id = 200 AND r.teleports = 0) AS kzt_pro,
		  SUM(r.mode_id = 201 AND r.teleports > 0) AS skz_tp,
		  SUM(r.mode_id = 201 AND r.teleports = 0) AS skz_pro,
		  SUM(r.mode_id = 202 AND r.teleports > 0) AS vnl_tp,
		  SUM(r.mode_id = 202 AND r.teleports = 0) AS vnl_pro
		FROM players AS p
		JOIN (
		  SELECT
		    r.*,
		    CASE WHEN r.teleports = 0 THEN 0 ELSE 1 END AS has_teleports
		  FROM records AS r
		  JOIN courses AS c ON c.id = r.course_id
		  JOIN maps AS m ON m.id = c.map_id
		  WHERE c.stage = 0
		  AND player_id =
		"#,
	);

	query
		.push_bind(player.id)
		.push(
			r#"
			  GROUP BY r.mode_id, r.course_id, has_teleports
			) AS r ON r.player_id = p.id
			AND p.id =
			"#,
		)
		.push_bind(player.id);

	let result = query
		.build_query_as::<CompletionQuery>()
		.fetch_one(&pool)
		.await
		.map(|query| Completion {
			id: query.id,
			name: query.name,
			is_banned: query.is_banned > 0,
			kzt_tp: query.kzt_tp.try_into().unwrap(),
			kzt_pro: query.kzt_pro.try_into().unwrap(),
			skz_tp: query.skz_tp.try_into().unwrap(),
			skz_pro: query.skz_pro.try_into().unwrap(),
			vnl_tp: query.vnl_tp.try_into().unwrap(),
			vnl_pro: query.vnl_pro.try_into().unwrap(),
		})?;

	debug!("> {result:#?}");

	Ok(Json(ResponseBody {
		result,
		took: (Instant::now() - start).as_nanos(),
	}))
}
