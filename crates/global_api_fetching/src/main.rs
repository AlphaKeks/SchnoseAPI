#![deny(clippy::complexity, clippy::perf)]
#![warn(clippy::suspicious, clippy::style)]

mod maps;
mod modes;
mod players;
mod records;
mod servers;

use {
	clap::{Parser, Subcommand},
	color_eyre::{eyre::eyre, Result as Eyre},
	log::{debug, info},
	std::{
		fs::File,
		io::{BufWriter, ErrorKind::NotFound, Write},
		path::PathBuf,
	},
};

#[derive(Debug, Parser)]
struct Args {
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

	/// Which data to fetch
	#[command(subcommand)]
	data_type: DataType,
}

#[tokio::main]
async fn main() -> Eyre<()> {
	let start = chrono::Utc::now().timestamp_millis();

	// setup error handling
	color_eyre::install()?;

	// parse cli args
	let args = Args::parse();

	// setup logging
	std::env::set_var(
		"RUST_LOG",
		if args.quiet { "global_api_fetching=ERROR" } else { "global_api_fetching=INFO" },
	);
	if args.debug {
		std::env::set_var("RUST_LOG", "global_api_fetching=DEBUG");
	}
	env_logger::init();

	let output_file = match File::options()
		.write(true)
		.open(&args.output_path)
	{
		Ok(file) => file,
		Err(why) => match why.kind() {
			NotFound => File::create(&args.output_path)?,
			why => return Err(eyre!("{why:?}")),
		},
	};
	let mut buf_writer = BufWriter::new(output_file);

	let gokz_client = gokz_rs::Client::new();

	info!("Fetching {}...", &args.data_type);
	match args.data_type {
		DataType::Maps { id } => maps::fetch(id, &mut buf_writer, &gokz_client).await?,
		DataType::Modes { id } => modes::fetch(id, &mut buf_writer, &gokz_client).await?,
		DataType::Servers { id } => servers::fetch(id, &mut buf_writer, &gokz_client).await?,
		DataType::Players {
			start_offset,
			chunk_size,
			backwards,
			limit,
		} => {
			players::fetch(
				start_offset,
				chunk_size,
				backwards,
				limit,
				&mut buf_writer,
				&gokz_client,
			)
			.await?
		}
		DataType::Records {
			start_id,
			backwards,
			limit,
		} => records::fetch(start_id, backwards, limit, &mut buf_writer, &gokz_client).await?,
	}

	let took = chrono::Utc::now().timestamp_millis() - start;
	info!("Finished after {:.3} seconds.", took as f64 / 1000.0);
	Ok(())
}

#[derive(Debug, Clone, Subcommand)]
enum DataType {
	/// `/maps`
	Maps {
		/// Fetch a single map by ID instead of all maps.
		#[arg(long)]
		id: Option<u16>,
	},

	/// `/modes`
	Modes {
		/// Fetch a single mode by ID instead of all modes.
		#[arg(long)]
		id: Option<u16>,
	},

	/// `/servers`
	Servers {
		/// Fetch a single server by ID instead of all servers.
		#[arg(long)]
		id: Option<u16>,
	},

	/// `/players`
	Players {
		#[arg(long = "offset")]
		#[clap(default_value = "0")]
		start_offset: usize,

		/// How many players to fetch at once. Max is 500.
		#[arg(long = "size")]
		#[clap(default_value = "500")]
		chunk_size: usize,

		/// Increase the offset instead of decreasing it each iteration.
		#[arg(short, long)]
		#[clap(default_value = "false")]
		backwards: bool,

		/// How many players to fetch before stopping.
		#[arg(short, long)]
		#[clap(default_value = "500")]
		limit: usize,
	},

	/// `/records`
	Records {
		/// The ID to start at.
		#[arg(long = "start")]
		#[clap(default_value = "0")]
		start_id: usize,

		/// Increase the offset instead of decreasing it each iteration.
		#[arg(short, long)]
		#[clap(default_value = "false")]
		backwards: bool,

		/// How many records to fetch before stopping.
		#[arg(short, long)]
		#[clap(default_value = "500")]
		limit: usize,
	},
}

impl std::fmt::Display for DataType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Maps { .. } => "maps",
				Self::Modes { .. } => "modes",
				Self::Servers { .. } => "servers",
				Self::Players { .. } => "players",
				Self::Records { .. } => "records",
			}
		)
	}
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
