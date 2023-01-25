use {
	crate::{
		output::{get_file, write_to_file},
		OutputMethod,
	},
	color_eyre::Result as Eyre,
	gokz_rs::{GlobalAPI, KZGO},
	log::{debug, info},
	serde::{Deserialize, Serialize},
	sqlx::{MySql, Pool},
	std::io::BufWriter,
};

pub(crate) async fn fetch_maps(
	output_method: OutputMethod,
	output_path: Option<String>,
	_table_name: Option<String>,
	_connection: Option<Pool<MySql>>,
) -> Eyre<()> {
	let client = gokz_rs::Client::new();

	let mut global_api_maps = GlobalAPI::get_maps(true, Some(9999), &client).await?;
	info!("Fetched GlobalAPI maps.");
	debug!("> {} maps", global_api_maps.len());

	let kzgo_maps = KZGO::get_maps(&client).await?;
	info!("Fetched KZ:GO maps.");
	debug!("> {} maps", kzgo_maps.len());

	assert_eq!(global_api_maps.len(), kzgo_maps.len());

	global_api_maps.sort_unstable_by(|a, b| a.id.cmp(&b.id));

	let maps: Vec<MergedMap> = global_api_maps
		.into_iter()
		.map(|global| {
			let kzgo = kzgo_maps
				.iter()
				.find(|map| map.name.as_ref().eq(&Some(&global.name)))
				.unwrap_or_else(|| panic!("Couldn't find `{}`.", &global.name));

			let mapper_id = kzgo.mapperIds[0].as_ref().unwrap().parse::<u64>().unwrap();

			MergedMap {
				id: global.id as u16,
				name: global.name,
				difficulty: global.difficulty as u8,
				validated: global.validated,
				filesize: global.filesize as u64,
				// HACK: Some mapper ids are way too low (e.g. 21, 37, ...) so we just replace
				// those with 0.
				created_by: if mapper_id < 76000000000000000 { 0 } else { mapper_id },
				approved_by: global.approved_by_steamid64.parse::<u64>().unwrap(),
				created_on: global.created_on,
				updated_on: global.updated_on,
			}
		})
		.collect();

	match output_method {
		OutputMethod::Json => {
			let output_path =
				output_path.expect("`output_path` is a required flag for this method.");

			let mut json = serde_json::to_vec(&maps)?;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MergedMap {
	id: u16,
	name: String,
	difficulty: u8,
	validated: bool,
	filesize: u64,
	created_by: u64,
	approved_by: u64,
	created_on: String,
	updated_on: String,
}
