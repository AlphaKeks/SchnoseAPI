use {
	crate::output::{get_file, write_to_file},
	color_eyre::Result as Eyre,
	gokz_rs::GlobalAPI,
	log::{debug, info},
	std::io::BufWriter,
};

pub(crate) async fn fetch_modes(output_path: Option<String>) -> Eyre<()> {
	let client = gokz_rs::Client::new();

	let modes = GlobalAPI::get_modes(&client).await?;
	info!("Fetched GlobalAPI modes.");
	debug!("> {} modes", modes.len());

	let output_path = output_path.unwrap_or_else(|| String::from("./modes.json"));

	let mut json = serde_json::to_vec(&modes)?;
	json.push(b'\n');
	let output_file = get_file(&output_path)?;
	let mut buf_writer = BufWriter::new(output_file);
	write_to_file(&mut buf_writer, &json, &output_path)?;

	Ok(())
}
