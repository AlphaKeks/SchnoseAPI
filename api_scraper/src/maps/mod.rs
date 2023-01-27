use {
	crate::output::{get_file, write_to_file},
	api_scraper::MergedMap,
	color_eyre::Result as Eyre,
	gokz_rs::{GlobalAPI, KZGO},
	log::{debug, info},
	std::io::BufWriter,
};

pub(crate) async fn fetch_maps(output_path: Option<String>) -> Eyre<()> {
	let client = gokz_rs::Client::new();

	let mut global_api_maps = GlobalAPI::get_maps(true, Some(9999), &client).await?;
	let mut non_global_maps = GlobalAPI::get_maps(false, Some(9999), &client).await?;
	global_api_maps.append(&mut non_global_maps);
	info!("Fetched GlobalAPI maps.");
	debug!("> {} maps", global_api_maps.len());

	let kzgo_maps = KZGO::get_maps(&client).await?;
	info!("Fetched KZ:GO maps.");
	debug!("> {} maps", kzgo_maps.len());

	// assert_eq!(global_api_maps.len(), kzgo_maps.len());

	global_api_maps.sort_unstable_by(|a, b| a.id.cmp(&b.id));

	let maps: Vec<MergedMap> = global_api_maps
		.into_iter()
		.map(|global| {
			let mapper_id = kzgo_maps
				.iter()
				.find(|map| map.name.as_ref().eq(&Some(&global.name)))
				.map_or(0, |map| map.mapperIds[0].as_ref().unwrap().parse::<u64>().unwrap());

			MergedMap {
				id: global.id as u16,
				name: global.name,
				difficulty: global.difficulty as u8,
				validated: global.validated,
				filesize: global.filesize as u64,
				// HACK: Some mapper ids are way too low (e.g. 21, 37, ...) so we just replace
				// those with 0.
				created_by: if mapper_id < 76_000_000_000_000_000 { 0 } else { mapper_id },
				approved_by: global.approved_by_steamid64.parse::<u64>().unwrap(),
				created_on: global.created_on,
				updated_on: global.updated_on,
			}
		})
		.collect();

	let output_path = output_path.unwrap_or_else(|| String::from("./maps.json"));

	let mut json = serde_json::to_vec(&maps)?;
	json.push(b'\n');
	let output_file = get_file(&output_path)?;
	let mut buf_writer = BufWriter::new(output_file);
	write_to_file(&mut buf_writer, &json, &output_path)?;

	Ok(())
}
