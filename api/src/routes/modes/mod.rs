use serde::Serialize;

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
	pub created_on: String,
}
