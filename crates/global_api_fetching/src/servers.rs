use {
	super::write_to_file,
	color_eyre::Result as Eyre,
	gokz_rs::GlobalAPI,
	log::debug,
	std::io::{BufWriter, Write},
};

pub async fn fetch<W: Write>(
	id: Option<u16>,
	buf_writer: &mut BufWriter<W>,
	gokz_client: &gokz_rs::Client,
) -> Eyre<()> {
	if let Some(id) = id {
		let server = GlobalAPI::get_server_by_id(id as i32, gokz_client).await?;
		debug!("Got server `{id}`: {server:?}");

		let json = serde_json::to_string(&server)?;
		return write_to_file(json.as_bytes(), buf_writer, true);
	}

	let servers = GlobalAPI::get_servers(Some(99999), gokz_client).await?;
	debug!("Got {} servers...", servers.len());

	let json = serde_json::to_vec(&servers)?;
	write_to_file(&json, buf_writer, true)?;

	Ok(())
}
