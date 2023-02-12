#![deny(clippy::complexity, clippy::perf)]
#![warn(clippy::suspicious, clippy::style)]

use {
	clap::Parser,
	color_eyre::Result as Eyre,
	log::{debug, error, info},
	std::{
		fs::File,
		io::{BufWriter, ErrorKind::NotFound, Write},
		path::PathBuf,
	},
};

#[derive(Debug, Parser)]
struct Args {
	/// Path to the input JSON file. Defaults to `./input.json`
	#[arg(short, long)]
	#[clap(default_value = "./input.json")]
	input_path: PathBuf,

	/// Print no logs to stdout.
	#[arg(short, long)]
	#[clap(default_value = "false")]
	quiet: bool,

	/// How many records per file
	#[arg(long)]
	chunk_size: usize,

	/// Print debug information. This option overrides `quiet`.
	#[arg(long)]
	#[clap(default_value = "false")]
	debug: bool,
}

fn main() -> Eyre<()> {
	let start = chrono::Utc::now().timestamp_millis();

	// setup error handling
	color_eyre::install()?;

	// parse cli args
	let args = Args::parse();

	// setup logging
	std::env::set_var(
		"RUST_LOG",
		if args.quiet { "split_files=ERROR" } else { "split_files=INFO" },
	);
	if args.debug {
		std::env::set_var("RUST_LOG", "split_files=DEBUG");
	}
	env_logger::init();

	let input_file = std::fs::read_to_string(&args.input_path)?;

	let input = serde_json::from_str::<Vec<Record>>(&input_file)?;
	for (i, records) in input
		.chunks(args.chunk_size)
		.enumerate()
	{
		let output_path = format!("{}_out_{}.json", args.input_path.to_string_lossy(), i + 1);
		let output_file = match File::options()
			.write(true)
			.open(&output_path)
		{
			Ok(file) => file,
			Err(why) => match why.kind() {
				NotFound => File::create(&output_path)?,
				_ => {
					error!("failed to do file stuff {why:?}");
					continue;
				}
			},
		};

		let mut buf_writer = BufWriter::new(output_file);

		let out_vec = match serde_json::to_vec(&records) {
			Ok(bytes) => bytes,
			Err(why) => {
				error!("failed to parse json {why:?}");
				continue;
			}
		};

		if let Err(why) = write_to_file(&out_vec, &mut buf_writer, true) {
			error!("failed to write to file {why:?}");
			continue;
		}
	}

	let took = chrono::Utc::now().timestamp_millis() - start;
	info!("Finished after {:.3} seconds.", took as f64 / 1000.0);
	Ok(())
}

fn write_to_file<W: Write>(data: &[u8], buf_writer: &mut BufWriter<W>, flush: bool) -> Eyre<()> {
	buf_writer.write_all(data)?;
	debug!("Wrote {} bytes to disk.", data.len());
	if flush {
		buf_writer.flush()?;
		debug!("Flushed BufWriter.");
	}
	Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Record {
	id: u32,
	steamid64: u64,
	player_name: String,
	steam_id: String,
	server_name: String,
	map_name: String,
	stage: u8,
	mode: String,
	tickrate: u8,
	time: f64,
	teleports: u32,
	created_on: String,
}
