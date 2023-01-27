use {
	axum::{http::StatusCode, response::IntoResponse, Json},
	log::{error, warn},
	serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Error {
	Unknown,
	Unexpected { expected: String },
	MySQL { msg: String },
}

impl IntoResponse for Error {
	fn into_response(self) -> axum::response::Response {
		match self {
			Self::Unknown => {
				(StatusCode::INTERNAL_SERVER_ERROR, Json(String::from("Unknown Error occurred.")))
			},
			Self::Unexpected { expected } => (StatusCode::INTERNAL_SERVER_ERROR, Json(expected)),
			Self::MySQL { msg } => (StatusCode::INTERNAL_SERVER_ERROR, Json(msg)),
		}
		.into_response()
	}
}

impl From<sqlx::Error> for Error {
	fn from(value: sqlx::Error) -> Self {
		warn!("{:?}", &value);

		match value {
			sqlx::Error::Database(db_err) => Self::MySQL { msg: String::from((*db_err).message()) },
			sqlx::Error::RowNotFound => {
				Self::MySQL { msg: String::from("No database entries found.") }
			},
			sqlx::Error::ColumnDecode { index, source } => {
				error!("Failed to decode column.");
				error!("{index}");
				error!("{source:?}");
				Self::Unexpected { expected: String::from("Failed to decode database column.") }
			},
			sqlx::Error::Decode(db_err) => {
				error!("Failed to decode value.");
				error!("{db_err:?}");
				Self::Unexpected { expected: String::from("Failed to decode database value.") }
			},
			_ => Self::Unknown,
		}
	}
}
