use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIResponse<T> {
	pub result: T,
	pub took: i64,
}
