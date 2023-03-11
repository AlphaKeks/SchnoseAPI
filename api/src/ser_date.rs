use {
	serde::{Serialize, Serializer},
	sqlx::types::time::PrimitiveDateTime,
};

pub(crate) fn ser_date<S>(date: &PrimitiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let date = date.to_string();
	let (date, _) = date
		.split_once('.')
		.ok_or(serde::ser::Error::custom("Invalid date format."))?;

	// to match the GlobalAPI's format
	date.replace(' ', "T")
		.serialize(serializer)
}
