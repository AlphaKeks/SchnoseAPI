use {database::schemas::FancyPlayer, serde::Serialize, sqlx::FromRow};

mod index;
pub(crate) use index::get as index;

mod ident;
pub(crate) use ident::get as ident;

#[derive(Debug, FromRow)]
struct ServerQuery {
	pub id: u16,
	pub name: String,
	pub owner_id: u32,
	pub owner_name: String,
	pub owner_is_banned: bool,
	pub approver_id: u32,
	pub approver_name: String,
	pub approver_is_banned: bool,
}

#[derive(Debug, Serialize)]
pub struct Server {
	pub id: u16,
	pub name: String,
	pub owned_by: FancyPlayer,
	pub approved_by: FancyPlayer,
}
