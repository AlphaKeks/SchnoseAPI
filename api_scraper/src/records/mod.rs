use {
	crate::{
		output::{get_file, write_to_file},
		OutputMethod,
	},
	color_eyre::Result as Eyre,
	gokz_rs::GlobalAPI,
	log::{debug, info},
	sqlx::{MySql, Pool},
	std::{io::BufWriter, time::Duration},
};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn fetch_records(
	start_id: isize,
	backwards: bool,
	limit: u32,
	delay: u64,
	output_method: OutputMethod,
	output_path: Option<String>,
	_table_name: Option<String>,
	_connection: Option<Pool<MySql>>,
) -> Eyre<()> {
	let client = gokz_rs::Client::new();
	let delay = Duration::from_millis(delay);

	match output_method {
		OutputMethod::Json => {
			let output_path = output_path.unwrap_or_else(|| String::from("./records.json"));
			let output_file = get_file(&output_path)?;
			let mut buf_writer = BufWriter::new(output_file);
			write_to_file(&mut buf_writer, &[b'['], &output_path)?;

			let mut total = 0;
			let mut i = start_id;
			info!("Starting the requests...");
			loop {
				let Ok(record) = GlobalAPI::get_record(i as i32, &client).await else {
					info!("No new records...");
					std::thread::sleep(delay);
					continue;
				};
				debug!("{:?}", &record);

				total += 1;
				let mut json = serde_json::to_vec(&record)?;
				_ = json.remove(0);
				_ = json.pop();

				if (i - start_id).abs() == limit as isize {
					_ = json.pop();
					write_to_file(&mut buf_writer, &[b']'], &output_path)?;
					break;
				}

				write_to_file(&mut buf_writer, &json, &output_path)?;
				info!("{} records", total);

				if backwards {
					i -= 1;
				} else {
					i += 1;
				}
				std::thread::sleep(delay);
			}
		},
		OutputMethod::MySQL => {
			todo!();
		},
	}

	Ok(())
}
