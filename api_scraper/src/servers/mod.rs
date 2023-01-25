use {
	crate::{
		output::{get_file, write_to_file},
		OutputMethod,
	},
	color_eyre::Result as Eyre,
	gokz_rs::GlobalAPI,
	log::{debug, info},
	sqlx::{MySql, Pool},
	std::io::BufWriter,
};

pub(crate) async fn fetch_servers(
	output_method: OutputMethod,
	output_path: Option<String>,
	_table_name: Option<String>,
	_connection: Option<Pool<MySql>>,
) -> Eyre<()> {
	let client = gokz_rs::Client::new();

	let servers = GlobalAPI::get_servers(Some(9999), &client).await?;
	info!("Fetched GlobalAPI servers.");
	debug!("> {} servers", servers.len());
	debug!("Servers:\n{:?}", &servers);

	match output_method {
		OutputMethod::Json => {
			let output_path =
				output_path.expect("`output_path` is a required flag for this method.");

			let mut json = serde_json::to_vec(&servers)?;
			json.push(b'\n');
			let output_file = get_file(&output_path)?;
			let mut buf_writer = BufWriter::new(output_file);
			write_to_file(&mut buf_writer, &json, &output_path)?;
		},
		OutputMethod::MySQL => {
			todo!();
		},
	}

	Ok(())
}
