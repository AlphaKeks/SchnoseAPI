#![deny(clippy::complexity, clippy::perf)]
#![warn(clippy::suspicious, clippy::style)]

use {
	clap::Parser,
	color_eyre::{eyre::eyre, Result as Eyre},
	elasticsearch::{
		auth::Credentials,
		http::{
			request::JsonBody,
			transport::{SingleNodeConnectionPool, TransportBuilder},
			Url,
		},
		Elasticsearch, Scroll, ScrollParts, SearchParts,
	},
	gokz_rs::prelude::*,
	log::{debug, info},
	serde::{de::DeserializeOwned, Deserialize, Serialize},
	serde_json::{json, Value as JsonValue},
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

	/// How many elastic records to fetch.
	#[arg(short, long)]
	limit: Option<usize>,

	/// Path to the config file containing the elastic url and login info. Defaults to
	/// `./config.toml`
	#[arg(short, long)]
	#[clap(default_value = "./config.toml")]
	config_path: PathBuf,

	/// Print debug information. This option overrides `quiet`.
	#[arg(long)]
	#[clap(default_value = "false")]
	debug: bool,
}

const TIME_LIMIT: &str = "60m";

static mut QUIT: bool = false;

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
		if args.quiet { "elastic_fetching=ERROR" } else { "elastic_fetching=INFO" },
	);
	if args.debug {
		std::env::set_var("RUST_LOG", "elastic_fetching=DEBUG");
	}
	env_logger::init();

	let config_file =
		std::fs::read_to_string(&args.config_path).expect("Failed to find `config.toml`.");
	let config: Config = toml::from_str(&config_file)?;

	let credentials = Credentials::Basic(config.username, config.password);
	let url = Url::parse(&config.elastic_url)?;
	let pool = SingleNodeConnectionPool::new(url);
	let transport = TransportBuilder::new(pool)
		.auth(credentials)
		.build()?;
	let client = Elasticsearch::new(transport.clone());

	let chunk_size = args
		.limit
		.map_or(10_000, |n| n.min(10_000));

	let max_records = {
		match args.limit {
			Some(limit) => limit.to_string(),
			None => String::from("∞"),
		}
	};

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

	let mut total = 0;

	// Since we are building a json array from multiple iterations, we start with a leading `[` and
	// then add more and more objects as we go.
	write_to_file(&[b'['], &mut buf_writer, false)?;

	let (scroll_id, query) = elastic_query::<_, RawRecord>(
		json! {
			{
				"query": {
					"match_all": {}
				}
			}
		},
		chunk_size as i64,
		&client,
	)
	.await?;

	let mut query = query
		.into_iter()
		.filter_map(|raw| Record::try_from(raw).ok())
		.collect::<Vec<_>>();

	if let Ok(limit) = max_records.parse::<usize>() {
		if query.len() > limit {
			query.truncate(limit);
		}
	}

	let mut initial_json = serde_json::to_vec(&query)?;
	_ = initial_json.remove(0);
	_ = initial_json.pop();

	write_to_file(&initial_json, &mut buf_writer, false)?;
	info!("{} / {max_records}", query.len());

	ctrlc::set_handler(|| unsafe {
		QUIT = true;
	})?;

	if let Some(scroll_id) = scroll_id {
		loop {
			unsafe {
				if QUIT {
					break;
				}
			}

			let scroll = Scroll::new(&transport, ScrollParts::ScrollId(&scroll_id));
			let Ok(scroll_result) = elastic_query_with_scroll_id::<_, RawRecord>(
				json! {
					{
						"scroll": TIME_LIMIT,
						"scroll_id": &scroll_id
					}
				},
				scroll,
				&scroll_id,
			)
			.await else {
				info!("no records PogO");
				continue;
			};

			let mut new_records = scroll_result
				.into_iter()
				.filter_map(|raw| Record::try_from(raw).ok())
				.collect::<Vec<_>>();

			if new_records.is_empty() {
				info!("no records PogO");
				continue;
			}

			if let Ok(limit) = max_records.parse::<usize>() {
				if total + new_records.len() > limit {
					new_records.truncate(limit - total);
				}
			}

			total += new_records.len();

			let mut json = serde_json::to_vec(&new_records)?;
			_ = initial_json.remove(0);
			_ = initial_json.pop();
			json.push(b',');

			write_to_file(&json, &mut buf_writer, true)?;
			info!("{total} / {max_records}");

			if let Ok(limit) = max_records.parse::<usize>() {
				if total == limit {
					break;
				}
			}
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

#[derive(Debug, Serialize, Deserialize)]
struct Config {
	elastic_url: String,
	username: String,
	password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawRecord {
	created_on: String,
	id: u32,
	map_name: String,
	mode: String,
	player_name: Option<String>,
	server: Option<String>,
	server_name: Option<String>,
	stage: i32,
	steamid64: String,
	teleports: i32,
	tickrate: i32,
	time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Record {
	pub id: u32,
	pub steamid64: u64,
	pub player_name: String,
	pub steam_id: String,
	pub server_name: String,
	pub map_name: String,
	pub stage: u8,
	pub mode: String,
	pub tickrate: u8,
	pub time: f64,
	pub teleports: u32,
	pub created_on: String,
}

impl TryFrom<RawRecord> for Record {
	type Error = color_eyre::Report;

	fn try_from(value: RawRecord) -> Result<Self, Self::Error> {
		Ok(Self {
			id: value.id,
			steamid64: value.steamid64.parse()?,
			player_name: sanitize(
				value
					.player_name
					.ok_or(eyre!("no player name"))?,
			)?,
			steam_id: SteamID::from(value.steamid64.parse::<u64>()?).to_string(),
			server_name: sanitize(
				value.server.unwrap_or(
					value
						.server_name
						.ok_or(eyre!("no server name"))?,
				),
			)?,
			map_name: value.map_name,
			stage: u8::try_from(value.stage)?,
			mode: value.mode.parse::<Mode>()?.api(),
			tickrate: u8::try_from(value.tickrate)?,
			time: value.time,
			teleports: u32::try_from(value.teleports)?,
			created_on: value.created_on,
		})
	}
}

fn sanitize(input: String) -> Eyre<String> {
	if input == "Bad Steamid64" {
		Err(eyre!("bad player name"))
	} else {
		Ok(input.replace(['\'', '"', ',', '\\'], ""))
	}
}

async fn elastic_query<Query, Return>(
	query: Query,
	chunk_size: i64,
	client: &Elasticsearch,
) -> Eyre<(Option<String>, Vec<Return>)>
where
	Query: Serialize,
	Return: DeserializeOwned + std::fmt::Debug,
{
	let response = client
		.search(SearchParts::Index(&["kzrecords2"]))
		.from(0)
		.size(chunk_size)
		.scroll(TIME_LIMIT)
		.body(query)
		.send()
		.await?;

	let body = response.json::<JsonValue>().await?;
	let hits = &body["hits"]["hits"];
	let scroll_id = serde_json::from_value::<String>(body["_scroll_id"].clone()).ok();

	if !hits.is_array() {
		return Err(eyre!("Not an array: {:#?}", hits));
	}

	let hits = hits
		.as_array()
		.expect("should always be an array")
		.iter()
		.filter_map(|obj| {
			let source = &obj["_source"];
			serde_json::from_value::<Return>(source.to_owned()).ok()
		})
		.collect();

	Ok((scroll_id, hits))
}

async fn elastic_query_with_scroll_id<Query, Return>(
	query: Query,
	client: Scroll<'_, '_, JsonBody<Query>>,
	scroll_id: &str,
) -> Eyre<Vec<Return>>
where
	Query: Serialize,
	Return: DeserializeOwned + std::fmt::Debug,
{
	let response = client
		.scroll_id(scroll_id)
		.body(query)
		.send()
		.await?;

	let body = response.json::<JsonValue>().await?;
	let hits = &body["hits"]["hits"];

	if !hits.is_array() {
		return Err(eyre!("Not an array: {:#?}", hits));
	}

	let hits = hits
		.as_array()
		.expect("should always be an array")
		.iter()
		.filter_map(|obj| {
			let source = &obj["_source"];
			serde_json::from_value::<Return>(source.to_owned()).ok()
		})
		.collect();

	Ok(hits)
}
