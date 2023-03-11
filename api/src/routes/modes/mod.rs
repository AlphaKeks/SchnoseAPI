use {crate::ser_date::ser_date, serde::Serialize, sqlx::types::time::PrimitiveDateTime};

mod ident;
pub(crate) use ident::get as ident;

mod index;
pub(crate) use index::get as index;

#[derive(Debug, Serialize)]
pub struct Mode {
	pub id: u8,
	pub name: String,
	pub name_short: String,
	pub name_long: String,
	#[serde(serialize_with = "ser_date")]
	pub created_on: PrimitiveDateTime,
}
