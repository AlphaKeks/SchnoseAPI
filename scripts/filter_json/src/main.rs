#![deny(clippy::complexity, clippy::perf)]
#![warn(clippy::suspicious, clippy::style)]

use {
	clap::Parser,
	color_eyre::Result as Eyre,
	gokz_rs::prelude::SteamID,
	log::{debug, info},
	std::collections::HashSet,
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

	/// Path to the file with the ids. Defaults to `./ids.txt`
	#[arg(long = "ids")]
	#[clap(default_value = "./ids.txt")]
	id_list_path: PathBuf,

	/// Path to the output JSON file. Defaults to `./output.json`
	#[arg(short, long)]
	#[clap(default_value = "./output.json")]
	output_path: PathBuf,

	/// Print no logs to stdout.
	#[arg(short, long)]
	#[clap(default_value = "false")]
	quiet: bool,

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
		if args.quiet { "split_json_records=ERROR" } else { "split_json_records=INFO" },
	);
	if args.debug {
		std::env::set_var("RUST_LOG", "split_json_records=DEBUG");
	}
	env_logger::init();

	let mut id_list = std::fs::read_to_string(&args.id_list_path)?
		.lines()
		.filter_map(|x| x.parse::<u32>().ok())
		.collect::<HashSet<_>>();
	let input_file = std::fs::read_to_string(&args.input_path)?;
	let output_file = match File::options()
		.write(true)
		.open(&args.output_path)
	{
		Ok(file) => file,
		Err(why) => match why.kind() {
			NotFound => File::create(&args.output_path)?,
			_ => {
				panic!("failed to do file stuff {why:?}");
			}
		},
	};

	let mut buf_writer = BufWriter::new(output_file);

	let input = input_file
		.lines()
		.enumerate()
		.filter_map(
			|(i, record)| match serde_json::from_str::<ElasticRecord>(record) {
				Ok(record) => {
					let record = DatabaseRecord::try_from(record._source).ok()?;
					if id_list.remove(&record.id) {
						None
					} else {
						Some(record)
					}
				}
				Err(why) => {
					eprintln!("FAILED IN LINE {}: {:?}", i + 1, why);
					None
				}
			},
		)
		.collect::<Vec<_>>();
	let len = input.len();

	write_to_file(&[b'[', b'\n'], &mut buf_writer, false)?;
	for (i, record) in input.into_iter().enumerate() {
		let json =
			serde_json::to_string(&record).unwrap_or_else(|_| panic!("FAILED IN LINE {}", i + 1));
		write_to_file(json.as_bytes(), &mut buf_writer, false)?;
		if i + 1 != len {
			write_to_file(&[b','], &mut buf_writer, false)?;
		}
		write_to_file(&[b'\n'], &mut buf_writer, false)?;
	}
	write_to_file(&[b']'], &mut buf_writer, true)?;

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
pub struct ElasticRecord {
	_index: Option<String>,
	_type: Option<String>,
	_id: Option<String>,
	_score: Option<usize>,
	_source: Record,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Record {
	id: u32,
	steamid64: String,
	player_name: Option<String>,
	steam_id: Option<String>,
	server_name: Option<String>,
	server: Option<String>,
	map_name: String,
	stage: u8,
	mode: String,
	tickrate: Option<u8>,
	time: f64,
	teleports: u32,
	created_on: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseRecord {
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

impl TryFrom<Record> for DatabaseRecord {
	type Error = String;

	fn try_from(value: Record) -> Result<Self, Self::Error> {
		let Ok(steam_id64) = value.steamid64.parse::<u64>() else {
			return Err(String::from("FUCK"));
		};

		if steam_id64 == 0 {
			return Err(String::from("FUCK"));
		}

		let steam_id = SteamID::from(steam_id64);
		Ok(Self {
			id: value.id,
			steamid64: steam_id64,
			player_name: value
				.player_name
				.unwrap_or_else(|| String::from("unknown")),
			steam_id: steam_id.to_string(),
			server_name: value.server_name.unwrap_or(
				value
					.server
					.unwrap_or_else(|| String::from("unknown")),
			),
			map_name: value.map_name,
			stage: value.stage,
			mode: value.mode,
			tickrate: value.tickrate.unwrap_or(128),
			time: value.time,
			teleports: value.teleports,
			created_on: value.created_on,
		})
	}
}
