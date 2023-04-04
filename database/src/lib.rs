use {
	serde::{Deserialize, Deserializer, Serialize, Serializer},
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

pub fn deserialize_date<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
	D: Deserializer<'de>,
{
	let date = String::deserialize(deserializer)?;

	Ok(DateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S")
		.map_err(|_| {
			serde::de::Error::invalid_value(
				serde::de::Unexpected::Other("Invalid date format"),
				&"Date with `%Y-%m-%dT%H:%M:%S` format",
			)
		})?
		.with_timezone(&Utc))
}

pub fn deserialize_date_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
	D: Deserializer<'de>,
{
	if let Some(date) = Option::<String>::deserialize(deserializer)? {
		return Ok(Some(
			DateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S")
				.map_err(|_| {
					serde::de::Error::invalid_value(
						serde::de::Unexpected::Other("Invalid date format"),
						&"Date with `%Y-%m-%dT%H:%M:%S` format",
					)
				})?
				.with_timezone(&Utc),
		));
	}

	Ok(None)
}

pub fn deserialize_bool<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
	D: Deserializer<'de>,
{
	Ok(u8::deserialize(deserializer)? == 1)
}
