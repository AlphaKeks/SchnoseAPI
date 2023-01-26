use {
	color_eyre::Result as Eyre,
	gokz_rs::servers::Server,
	log::{debug, info},
	sqlx::{MySql, Pool},
};

/// HACK: I don't actually know who approved the servers, and since this is a toy application
/// anyway I'm putting gosh here because he's very handsome
const BACKUP_SERVER_APPROVER: u64 = 76561198292029626;

pub(crate) async fn insert(
	input: Vec<Server>,
	chunk_size: u64,
	table_name: &str,
	database_connection: &Pool<MySql>,
) -> Eyre<()> {
	let total = input.len();
	debug!("> {} servers", total);

	for (i, modes) in input.chunks(chunk_size as usize).enumerate() {
		let sql_query = build_query(modes, table_name);
		sqlx::query(&sql_query).execute(database_connection).await?;
		info!("{} / {} rows. ({}%)", i, total, (i as f32 / total as f32) * 100.0);
	}

	Ok(())
}

fn build_query(servers: &[Server], table_name: &str) -> String {
	let Server { id, port: _, ip: _, name, owner_steamid64 } = &servers[0];

	let mut query = format!(
		r#"
INSERT INTO {table_name}
  (
    id,
    name,
    owner_id,
    approved_by
  )
VALUES
  (
    {id},
    "{name}",
    {owner_steamid64},
    {}
  )"#,
		BACKUP_SERVER_APPROVER
	);

	for Server { id, port: _, ip: _, name, owner_steamid64 } in servers.iter().skip(1) {
		query.push_str(&format!(
			r#"
 ,(
    {id},
    "{name}",
    {owner_steamid64},
    {}
  )"#,
			BACKUP_SERVER_APPROVER
		));
	}

	query
}
