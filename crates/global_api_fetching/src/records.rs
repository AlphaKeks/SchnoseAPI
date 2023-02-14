use {
	super::write_to_file,
	color_eyre::Result as Eyre,
	gokz_rs::GlobalAPI,
	log::{debug, info},
	std::{
		io::{BufWriter, Write},
		time::Duration,
	},
};

const DELAY: Duration = Duration::from_millis(800);

pub async fn fetch<W: Write>(
	start_id: usize,
	backwards: bool,
	limit: usize,
	buf_writer: &mut BufWriter<W>,
	gokz_client: &gokz_rs::Client,
) -> Eyre<()> {
	let mut record_id = start_id;

	let mut total = 0;
	// Since we are building a json array from multiple iterations, we start with a leading `[` and
	// then add more and more objects as we go.
	write_to_file(&[b'['], buf_writer, false)?;

	for i in 1.. {
		let Ok(record) = GlobalAPI::get_record(record_id as i32, gokz_client).await else {
			// Something went wrong during the request.
			// If we are scraping backwards, we either
			// 1. hit a hole => decrease `record_id`
			// 2. hit the last record => exit
			if backwards {
				if record_id == 0 {
					info!("Hit last record. Exiting...");
					break;
				}

				info!("Record #{record_id} doesn't exist...");
				std::thread::sleep(DELAY);
				record_id -= 1;
				continue;
			}

			// We hit the newest record and want to sleep for some time. We also want only 1 record
			// per request now.
			info!("No new records found. Sleeping {:.2}ms.", DELAY.as_millis());
			std::thread::sleep(DELAY);
			record_id += 1;
			continue;
		};

		debug!("[{i}] Record:\n{record:?}");

		total += 1;
		let mut json = serde_json::to_vec(&record)?;

		let last_iteration = total == limit;

		let flush = if last_iteration {
			// Append final `]` to finish our JSON array.
			json.push(b']');
			true
		} else {
			// Append trailing `,` in preparation for the next iteration.
			json.push(b',');
			false
		};

		write_to_file(&json, buf_writer, flush)?;
		info!("[{i}] {total} / {limit} records.");

		if last_iteration {
			// We're done.
			break;
		}

		if backwards {
			if record_id == 0 {
				break;
			}
			record_id -= 1;
		} else {
			record_id += 1;
		}
		std::thread::sleep(DELAY);
	}

	Ok(())
}
