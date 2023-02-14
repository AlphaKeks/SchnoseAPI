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
	start_offset: usize,
	chunk_size: usize,
	backwards: bool,
	limit: usize,
	buf_writer: &mut BufWriter<W>,
	gokz_client: &gokz_rs::Client,
) -> Eyre<()> {
	let mut offset = start_offset;
	let mut chunk_size = chunk_size;

	let mut total = 0;
	// Since we are building a json array from multiple iterations, we start with a leading `[` and
	// then add more and more objects as we go.
	write_to_file(&[b'['], buf_writer, false)?;

	for i in 1.. {
		let Ok(players_request) = GlobalAPI::get_players(Some(offset as i32), Some(chunk_size as u32), gokz_client).await else {
			// Something went wrong during the request.
			// If we are scraping backwards, we probably hit the last player and therefore want to
			// exit.
			if backwards {
				info!("No players left. Exiting...");
				break;
			}

			// We hit the newest player and want to sleep for some time. We also want only 1 player
			// per request now.
			info!("No new players found. Sleeping {:.2}ms.", DELAY.as_millis());
			std::thread::sleep(DELAY);
			chunk_size = 1;
			continue;
		};

		let mut filtered_players = players_request
			.into_iter()
			.filter(|player| player.name.ne("Bad Steamid64"))
			.collect::<Vec<_>>();

		if total + filtered_players.len() > limit {
			// The players we got with this request would overshoot the limit, so we truncate the
			// results.
			filtered_players.truncate(limit - total);
		}

		debug!("[{i}] Players:\n{filtered_players:?}");

		total += filtered_players.len();
		let mut json = serde_json::to_vec(&filtered_players)?;
		// Remove the `[]` at the start and end.
		_ = json.remove(0);
		_ = json.pop();

		let last_iteration = total >= limit;

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
		info!("[{i}] {total} / {limit} players.");

		if last_iteration {
			// We're done.
			break;
		}

		if backwards {
			// decrement the offset, but we don't want negative numbers.
			let new_offset = offset as isize - chunk_size as isize;
			if new_offset.is_negative() {
				offset = 0;
			} else {
				offset = new_offset as usize;
			}
		} else if chunk_size == 1 {
			// `chunk_size` is 1 if we couldn't find a player but also aren't going backwards. This
			// means that we probably hit the newest player and only want to increase out offset by
			// 1.
			offset += 1;
		} else {
			offset += chunk_size;
		}
		std::thread::sleep(DELAY);
	}

	Ok(())
}
