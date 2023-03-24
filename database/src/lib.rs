use {
	serde::{Serialize, Serializer},
	sqlx::types::time::PrimitiveDateTime,
};

pub mod insert;
pub mod schemas;
pub mod select;

pub fn format_date(date: &PrimitiveDateTime) -> String {
	let date = date.to_string();
	let (date, _) = date
		.split_once('.')
		.expect("Invalid date format.");

	date.replace(' ', "T")
}

pub fn serialize_date<S: Serializer>(
	date: &PrimitiveDateTime,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	format_date(date).serialize(serializer)
}
