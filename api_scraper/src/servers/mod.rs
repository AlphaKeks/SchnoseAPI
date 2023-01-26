use {
	crate::output::{get_file, write_to_file},
	color_eyre::Result as Eyre,
	gokz_rs::GlobalAPI,
	log::{debug, info},
	std::io::BufWriter,
};

pub(crate) async fn fetch_servers(output_path: Option<String>) -> Eyre<()> {
	let client = gokz_rs::Client::new();

	let mut servers = GlobalAPI::get_servers(Some(9999), &client).await?;
	info!("Fetched GlobalAPI servers.");
	debug!("> {} servers", servers.len());
	debug!("Servers:\n{:?}", &servers);

	servers.sort_unstable_by(|a, b| a.id.cmp(&b.id));

	let output_path = output_path.unwrap_or_else(|| String::from("./servers.json"));

	let mut json = serde_json::to_vec(&servers)?;
	json.push(b'\n');
	let output_file = get_file(&output_path)?;
	let mut buf_writer = BufWriter::new(output_file);
	write_to_file(&mut buf_writer, &json, &output_path)?;

	Ok(())
}
