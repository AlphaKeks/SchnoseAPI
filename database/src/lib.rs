use {
	serde::{Serialize, Serializer},
	sqlx::types::chrono::{DateTime, Utc},
};

pub mod insert;
pub mod schemas;
pub mod select;

pub fn format_date(date: &DateTime<Utc>) -> String {
	date.format("%Y-%m-%dT%H:%M:%S")
		.to_string()
}

pub fn serialize_date<S: Serializer>(
	date: &DateTime<Utc>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	format_date(date).serialize(serializer)
}
