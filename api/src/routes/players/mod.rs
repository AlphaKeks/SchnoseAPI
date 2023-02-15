use {
	crate::GlobalState,
	axum::{routing::get, Router},
};

mod index;
pub(crate) use index::get as index;

mod id;
pub(crate) use id::get as id;

pub(crate) fn router() -> Router<GlobalState> {
	Router::new()
		.route("/players", get(index))
		.route("/players/:id", get(id))
}
