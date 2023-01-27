use {
	crate::{
		models::{
			records::{RecordResponse, RecordsQuery},
			APIResponse, Error,
		},
		GlobalState,
	},
	axum::{
		extract::{Query, State},
		Json,
	},
};

pub async fn recent(
	query: Query<RecordsQuery>,
	state: State<GlobalState>,
) -> Result<Json<APIResponse<Vec<RecordResponse>>>, Error> {
	super::_index(query, state, true).await
}
