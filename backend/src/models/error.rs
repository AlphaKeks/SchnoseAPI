use {
	axum::{http::StatusCode, response::IntoResponse, Json},
	log::error,
	serde::Serialize,
	std::fmt::Display,
};

/// Global Result type for all handler functions
pub type Result<T> = std::result::Result<T, Error>;

/// Global return type for all handler functions
#[derive(Debug, Serialize)]
pub struct Response<T> {
	pub result: T,
	pub took: u128,
}

/// Global error type for all handler functions
#[derive(Debug, Clone, Serialize)]
pub enum Error {
	/// Something happened which was not supposed to happen. (This is reserved for edge cases)
	Unknown,

	/// An error that is expected to happen _sometimes_ but still an edge case that doesn't deserve
	/// its own variant.
	Custom { message: String },

	/// An error occurred when interacting with the database.
	Database { kind: DatabaseError },

	/// An error from the [`gokz_rs`] crate.
	GOKZ { message: String },

	/// Failed to parse Json.
	Json { message: Option<String> },
}

/// The different kinds of database errors that can occurr.
#[derive(Debug, Clone, Serialize)]
pub enum DatabaseError {
	Access,
	NoRows,
	Other,
}

impl std::error::Error for Error {}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Error::Unknown => "Some unknown error occurred. Please report this incident on GitHub.",
			Error::Custom { message } => message.as_str(),
			Error::Database { kind } => match kind {
				DatabaseError::Access => {
					"Failed to access database. Please report this incident on GitHub."
				}
				DatabaseError::NoRows => "Found no data.",
				DatabaseError::Other => "Database error. Please report this incident on GitHub.",
			},
			Error::GOKZ { message } => message.as_str(),
			Error::Json { message } => {
				if let Some(message) = message {
					message.as_str()
				} else {
					"Failed to parse Json."
				}
			}
		})
	}
}

impl IntoResponse for Error {
	fn into_response(self) -> axum::response::Response {
		let Self::Database { ref kind } = self else {
			return (StatusCode::INTERNAL_SERVER_ERROR, Json(self.to_string())).into_response();
		};

		if let DatabaseError::NoRows = kind {
			(StatusCode::NO_CONTENT, Json(self.to_string()))
		} else {
			(StatusCode::INTERNAL_SERVER_ERROR, Json(self.to_string()))
		}
		.into_response()
	}
}

impl From<sqlx::Error> for Error {
	fn from(value: sqlx::Error) -> Self {
		Self::Database {
			kind: match value {
				sqlx::Error::RowNotFound => DatabaseError::NoRows,
				why @ (sqlx::Error::Database(_)
				| sqlx::Error::Configuration(_)
				| sqlx::Error::Io(_)
				| sqlx::Error::Tls(_)
				| sqlx::Error::PoolTimedOut
				| sqlx::Error::PoolClosed
				| sqlx::Error::WorkerCrashed) => {
					error!("Failed to access database.\n{why:#?}");
					DatabaseError::Access
				}
				why => {
					error!("Database error occurred.\n{why:#?}");
					DatabaseError::Other
				}
			},
		}
	}
}

impl From<gokz_rs::Error> for Error {
	fn from(value: gokz_rs::Error) -> Self {
		Self::GOKZ {
			message: value.to_string(),
		}
	}
}
