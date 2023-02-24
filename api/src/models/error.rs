use {
	axum::{
		http::StatusCode,
		response::{IntoResponse, Response},
		Json,
	},
	chrono::ParseError,
	// log::error,
	log::warn,
	serde::{Deserialize, Serialize},
	sqlx::Error as SQLError,
	std::fmt::Display,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum Error {
	Unknown,
	Infallible,
	Custom { message: String },
	Database { message: String },
	GOKZ { message: String },
	Input { message: String, expected: String },
	JSON,
	Date,
	DateRange,
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&match self {
			Self::Unknown => String::from("Unknown error occurred."),
			Self::Infallible => String::from("Encountered error which should not have happened."),
			Self::Custom { message } | Error::Database { message } | Error::GOKZ { message } => {
				message.to_owned()
			}
			Self::Input { message, expected } => format!("{message} Expected `{expected}`."),
			Self::JSON => String::from("Failed to parse JSON."),
			Self::Date => String::from("Invalid Date format."),
			Self::DateRange => String::from("Invalid Date range."),
		})
	}
}

impl IntoResponse for Error {
	fn into_response(self) -> Response {
		match self {
			Self::Input { .. } => (StatusCode::BAD_REQUEST, Json(self.to_string())),
			Self::Database { message } => (StatusCode::NO_CONTENT, Json(message)),
			_ => (StatusCode::INTERNAL_SERVER_ERROR, Json(self.to_string())),
		}
		.into_response()
	}
}

impl From<sqlx::Error> for Error {
	fn from(value: sqlx::Error) -> Self {
		warn!("SQL Error: {value:?}");
		if let SQLError::RowNotFound = value {
			Self::Database {
				message: String::from("No entries found."),
			}
		} else {
			Self::Database {
				message: String::from("Database error."),
			}
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
		Self::Infallible
	}
}

impl From<color_eyre::Report> for Error {
	fn from(value: color_eyre::Report) -> Self {
		Self::Custom {
			message: value.to_string(),
		}
	}
}

impl From<ParseError> for Error {
	fn from(_: ParseError) -> Self {
		Self::Date
	}
}
