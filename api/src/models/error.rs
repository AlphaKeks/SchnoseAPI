use {
	axum::{
		http::StatusCode,
		response::{IntoResponse, Response},
		Json,
	},
	log::{error, warn},
	serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum Error {
	Unknown,
	Custom { message: String },
	Database { message: String },
	GOKZ { message: String },
	Input { message: String, expected: String },
}

impl IntoResponse for Error {
	fn into_response(self) -> Response {
		match self {
			Error::Unknown => (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(String::from("Unknown error occurred.")),
			),
			Error::Custom { message } | Error::Database { message } | Error::GOKZ { message } => {
				(StatusCode::INTERNAL_SERVER_ERROR, Json(message))
			}
			Error::Input { message, expected } => (
				StatusCode::BAD_REQUEST,
				Json(format!("{message} Expected `{expected}`.")),
			),
		}
		.into_response()
	}
}

impl From<sqlx::Error> for Error {
	fn from(value: sqlx::Error) -> Self {
		warn!("SQL Error: {value:?}");
		match value {
			sqlx::Error::Database(db_err) => Self::Database {
				message: String::from((*db_err).message()),
			},
			sqlx::Error::RowNotFound => Self::Database {
				message: String::from("No entries found."),
			},
			sqlx::Error::ColumnDecode { index, source } => {
				error!("Failed to decode column.");
				error!("{index:?}");
				error!("{source:?}");
				Self::Database {
					message: String::from("Failed to decode database column."),
				}
			}
			sqlx::Error::Decode(db_err) => {
				error!("Failed to decode value.");
				error!("{db_err:?}");
				Self::Database {
					message: String::from("Failed to decode database value."),
				}
			}
			_ => Self::Unknown,
		}
	}
}

impl From<gokz_rs::prelude::Error> for Error {
	fn from(value: gokz_rs::prelude::Error) -> Self {
		warn!("GOKZ Error: {value:?}");
		Self::GOKZ { message: value.msg }
	}
}

impl From<std::convert::Infallible> for Error {
	fn from(_: std::convert::Infallible) -> Self {
		Self::Unknown
	}
}

impl From<color_eyre::Report> for Error {
	fn from(value: color_eyre::Report) -> Self {
		Self::Custom {
			message: value.to_string(),
		}
	}
}
