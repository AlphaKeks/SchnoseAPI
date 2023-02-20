//! Because I don't have a better name for it.

use serde::{de::Error, Deserialize, Deserializer};

pub fn number_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
	D: Deserializer<'de>,
{
	let num = i32::deserialize(deserializer)?;
	if num == 1 {
		Ok(true)
	} else if num == 0 {
		Ok(false)
	} else {
		Err(Error::custom(crate::Error::JSON))
	}
}
