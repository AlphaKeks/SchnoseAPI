use {
	crate::write_to_file,
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
		let map = GlobalAPI::get_map(&MapIdentifier::ID(id as i32), gokz_client).await?;
		debug!("Got map `{id}`: {map:?}");

		let json = serde_json::to_string(&map)?;
		return write_to_file(json.as_bytes(), buf_writer, true);
	}

	let mut global_maps = GlobalAPI::get_maps(true, Some(99999), gokz_client).await?;
	debug!("Got {} global maps...", global_maps.len());

	let non_global_maps = GlobalAPI::get_maps(false, Some(99999), gokz_client).await?;
	debug!("Got {} non-global maps...", non_global_maps.len());

	global_maps.extend(non_global_maps);

	let json = serde_json::to_vec(&global_maps)?;
	write_to_file(&json, buf_writer, true)?;

	Ok(())
}
