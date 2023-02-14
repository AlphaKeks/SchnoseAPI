use {
	super::write_to_file,
	color_eyre::Result as Eyre,
	gokz_rs::{prelude::*, GlobalAPI},
	log::debug,
	std::io::{BufWriter, Write},
};

pub async fn fetch<W: Write>(
	id: Option<u16>,
	buf_writer: &mut BufWriter<W>,
	gokz_client: &gokz_rs::Client,
) -> Eyre<()> {
	if let Some(id) = id {
		let mode = GlobalAPI::get_mode(Mode::try_from(id as u8)?, gokz_client).await?;
		debug!("Got mode `{id}`: {mode:?}");

		let json = serde_json::to_string(&mode)?;
		return write_to_file(json.as_bytes(), buf_writer, true);
	}

	let modes = GlobalAPI::get_modes(gokz_client).await?;
	debug!("Got {} modes...", modes.len());

	let json = serde_json::to_vec(&modes)?;
	write_to_file(&json, buf_writer, true)?;

	Ok(())
}
