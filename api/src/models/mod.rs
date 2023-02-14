use {
	crate::Error,
	axum::Json,
	serde::{Deserialize, Serialize},
};

pub(crate) mod error;

pub(crate) type Response<T> = Result<Json<ResponseBody<T>>, Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ResponseBody<T> {
	pub(crate) result: T,
	pub(crate) took: f64,
}
