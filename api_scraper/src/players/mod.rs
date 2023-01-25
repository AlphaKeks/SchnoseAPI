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
pub(crate) async fn fetch_players(
	start_offset: i32,
	chunk_size: u32,
	backwards: bool,
	limit: u32,
	delay: u64,
	output_method: OutputMethod,
	output_path: Option<String>,
	_table_name: Option<String>,
	_connection: Option<Pool<MySql>>,
) -> Eyre<()> {
	let client = gokz_rs::Client::new();

	// 3102361
	let mut offset = start_offset;
	let delay = Duration::from_millis(delay);

	match output_method {
		OutputMethod::Json => {
			let mut total = 0;
			let output_path =
				output_path.expect("`output_path` is a required flag for this method.");
			let output_file = get_file(&output_path)?;
			let mut buf_writer = BufWriter::new(output_file);
			write_to_file(&mut buf_writer, &[b'['], &output_path)?;

			info!("Starting the requests...");
			for i in 1.. {
				let Ok(player_req) = GlobalAPI::get_players(Some(offset), Some(chunk_size), &client).await else {
					info!("No new players...");
					std::thread::sleep(delay);
					continue;
				};

				let player_req = player_req
					.into_iter()
					.filter(|player| player.name.ne("Bad Steamid64"))
					.collect::<Vec<_>>();

				debug!("{:?}", &player_req);

				total += player_req.len();
				let mut json = serde_json::to_vec(&player_req)?;
				_ = json.remove(0);
				_ = json.pop();
				if i * chunk_size != limit {
					json.push(b',');
				}
				write_to_file(&mut buf_writer, &json, &output_path)?;
				info!("{} iterations, {} players", i, total);

				if i * chunk_size == limit {
					write_to_file(&mut buf_writer, &[b']'], &output_path)?;
					break;
				}

				if backwards {
					offset += chunk_size as i32;
				} else {
					offset -= chunk_size as i32;
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
