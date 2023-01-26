use {
	clap::Parser,
	color_eyre::eyre::eyre,
	color_eyre::Result as Eyre,
	elasticsearch::{
		auth::Credentials,
		http::{
			request::JsonBody,
			transport::{SingleNodeConnectionPool, TransportBuilder},
			Url,
		},
		Elasticsearch, Scroll, ScrollParts, SearchParts,
	},
	gokz_rs::{prelude::*, GlobalAPI},
	log::info,
	serde::de::DeserializeOwned,
	serde::{Deserialize, Serialize},
	serde_json::{json, Value as JsonValue},
	sqlx::{mysql::MySqlPoolOptions, MySql, Pool},
	std::collections::HashMap,
};

/// CLI tool to fetch records from LoB's elastic database and insert them into a MySQL database.
#[derive(Debug, Parser)]

struct Args {
	/// MySQL table name to insert into.
	#[arg(short, long)]
	table_name: String,

	/// How many elastic records to fetch at once. Defaults to `1000`. Maximum is `10000`.
	#[arg(long)]
	elastic_chunk_size: Option<i64>,

	/// How many rows to insert at once. Defaults to `1000`.
	#[arg(long)]
	sql_chunk_size: Option<u64>,

	/// How long to keep the connection to elastic in minutes. Defaults to `15`.
	#[arg(short = 'l', long)]
	time_limit: Option<u64>,

	/// Path to the `config.toml` file containing the database connection string. See
	/// `config.toml.example`. Defaults to `./config.toml`.
	#[arg(long = "config")]
	config_path: Option<String>,

	/// Don't print any output. The `debug` flag overrides this flag.
	#[arg(short, long)]
	quiet: bool,

	/// Print debug information.
	#[arg(long)]
	debug: bool,
}

const DEFAULT_CUNK_SIZE: i64 = 1000;
const DEFAULT_CONFIG_PATH: &str = "./config.toml";

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;
	let args = Args::parse();

	std::env::set_var("RUST_LOG", "api_scraper=ERROR");

	if !args.quiet {
		std::env::set_var("RUST_LOG", "api_scraper=INFO");
	}

	if args.debug {
		std::env::set_var("RUST_LOG", "api_scraper=DEBUG");
	}

	env_logger::init();

	let config_path = args.config_path.unwrap_or_else(|| String::from(DEFAULT_CONFIG_PATH));
	let config_file = std::fs::read_to_string(&config_path)?;
	let config: Config = toml::from_str(&config_file)?;

	let credentials = Credentials::Basic(config.username, config.password);
	let url = Url::parse(&config.elastic_url)?;
	let pool = SingleNodeConnectionPool::new(url);
	let transport = TransportBuilder::new(pool).auth(credentials).build()?;
	let client = Elasticsearch::new(transport.clone());

	let (elastic_chunk_size, amount_of_searches) = match args.elastic_chunk_size {
		Some(chunk_size) if chunk_size > 10_000 => (10_000, chunk_size / 10_000),
		Some(chunk_size) => (chunk_size, 1),
		None => (DEFAULT_CUNK_SIZE, 1),
	};

	let time_limit = format!("{}m", args.time_limit.unwrap_or(60 * 60 * 24));

	let filter = |mut rec: RawRecord| {
		if let Some(ref mut player_name) = rec.player_name {
			if player_name == "Bad Steamid64" {
				return None;
			}
			*player_name = player_name.replace(['"', '\'', '\\'], "");
		}
		if let Some(ref mut server_name) = rec.server_name {
			*server_name = server_name.replace(['"', '\'', '\\'], "");
		}
		Some(rec)
	};

	let (id, query) = elastic_query::<RawRecord, _, _>(
		json! {
			{
				"query": {
					"match_all": {}
				}
			}
		},
		elastic_chunk_size,
		&client,
		Some(filter),
	)
	.await?;

	let conn = MySqlPoolOptions::new().max_connections(1).connect(&config.sql_url).await?;
	let records = query.into_iter().map(Record::from).collect::<Vec<_>>();
	let global_maps = GlobalAPI::get_maps(false, Some(9999), &gokz_rs::Client::new())
		.await?
		.into_iter()
		.map(|map| (map.name, map.id))
		.collect::<HashMap<_, _>>();
	let sql_query = build_query(&records, &args.table_name, &conn, &global_maps).await?;
	sqlx::query(&sql_query).execute(&conn).await?;
	info!("Inserted {} records.", records.len());

	if let Some(scroll_id) = id {
		for _ in 1..amount_of_searches {
			let scroll = Scroll::new(&transport, ScrollParts::ScrollId(&scroll_id));
			let scroll_result = elastic_query_with_scroll_id::<RawRecord, _, _>(
				json! {
					{
						"scroll": &time_limit,
						"scroll_id": &scroll_id
					}
				},
				scroll,
				&scroll_id,
				Some(filter),
			)
			.await?;

			let new_records: Vec<Record> = scroll_result
				.into_iter()
				.map(|raw| {
					let mut new = Record::from(raw);
					new.player_name = new.player_name.replace(['"', '\'', '\\'], "");
					new.server_name = new.server_name.replace(['"', '\'', '\\'], "");
					new
				})
				.collect();

			let sql_query =
				build_query(&new_records, &args.table_name, &conn, &global_maps).await?;
			sqlx::query(&sql_query).execute(&conn).await?;
			info!("Inserted {} records.", new_records.len());
		}
	};

	Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
	sql_url: String,
	elastic_url: String,
	username: String,
	password: String,
}
async fn elastic_query<T, Q, F>(
	query: Q,
	chunk_size: i64,
	client: &Elasticsearch,
	filter: Option<F>,
) -> Eyre<(Option<String>, Vec<T>)>
where
	T: DeserializeOwned + std::fmt::Debug,
	Q: Serialize,
	F: Fn(T) -> Option<T>,
{
	let response = client
		.search(SearchParts::Index(&["kzrecords2"]))
		.from(0)
		.size(chunk_size)
		.scroll("1m")
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
			let mut json = serde_json::from_value::<T>(source.to_owned()).ok()?;
			if let Some(filter) = &filter {
				json = filter(json)?;
			}
			Some(json)
		})
		.collect();

	Ok((scroll_id, hits))
}

async fn elastic_query_with_scroll_id<T, Q, F>(
	query: Q,
	client: Scroll<'_, '_, JsonBody<Q>>,
	scroll_id: &str,
	filter: Option<F>,
) -> Eyre<Vec<T>>
where
	T: DeserializeOwned + std::fmt::Debug,
	Q: Serialize,
	F: Fn(T) -> Option<T>,
{
	let response = client.scroll_id(scroll_id).body(query).send().await?;

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
			let mut json = serde_json::from_value::<T>(source.to_owned()).ok()?;
			if let Some(filter) = &filter {
				json = filter(json)?;
			}
			Some(json)
		})
		.collect();

	Ok(hits)
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

impl From<RawRecord> for Record {
	fn from(value: RawRecord) -> Self {
		let steam_id64: u64 = value.steamid64.parse().unwrap();
		Record {
			id: value.id,
			map_name: value.map_name,
			steam_id: SteamID::from(steam_id64).to_string(),
			player_name: value.player_name.unwrap_or_else(|| String::from("unknown")),
			steam_id64,
			mode: value.mode.parse::<Mode>().unwrap() as u8,
			stage: value.stage as u8,
			teleports: value.teleports as u32,
			time: value.time,
			server_name: value
				.server_name
				.unwrap_or_else(|| value.server.unwrap_or_else(|| String::from("unknown"))),
			created_on: chrono::NaiveDateTime::parse_from_str(
				&value.created_on,
				"%Y-%m-%dT%H:%M:%S",
			)
			.expect("Failed to parse date.")
			.timestamp() as u64,
		}
	}
}
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
struct Record {
	pub id: u32,
	pub map_name: String,
	pub steam_id: String,
	pub player_name: String,
	pub steam_id64: u64,
	pub mode: u8,
	pub stage: u8,
	pub teleports: u32,
	pub time: f64,
	pub server_name: String,
	pub created_on: u64,
}

async fn build_query(
	records: &[Record],
	table_name: &str,
	conn: &Pool<MySql>,
	global_maps: &HashMap<String, i32>,
) -> Eyre<String> {
	let Record {
		id,
		map_name,
		steam_id: _,
		player_name: _,
		steam_id64: steamid64,
		mode,
		stage,
		teleports,
		time,
		server_name,
		created_on,
	} = &records[0];

	let mode = Mode::try_from(*mode).unwrap();
	let mode_id = mode as u8;

	let ServerID { id: server_id } = sqlx::query_as::<_, ServerID>(&format!(
		r#"SELECT id FROM servers WHERE name = "{}" LIMIT 1"#,
		server_name
	))
	.fetch_one(conn)
	.await?;

	let map_id = global_maps.get(map_name).unwrap();

	let mut query = format!(
		r#"
INSERT INTO {table_name}
  (
    map_id,
    mode_id,
    player_id,
    server_id,
    stage,
    teleports,
    time,
    created_on,
    global_id
  )
VALUES
  (
    {map_id},
    {mode_id},
    {steamid64},
    {server_id},
    {stage},
    {teleports},
    {time},
    "{created_on}",
    {id}
  )"#
	);

	for Record {
		id,
		map_name: _,
		steam_id: _,
		player_name: _,
		steam_id64: _,
		mode: _,
		stage,
		teleports,
		time,
		server_name: _,
		created_on,
	} in records.iter().skip(1)
	{
		query.push_str(&format!(
			r#"
 ,(
    {map_id},
    {mode_id},
    {steamid64},
    {server_id},
    {stage},
    {teleports},
    {time},
    "{created_on}",
    {id}
  )"#
		));
	}

	Ok(query)
}

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
struct ServerID {
	id: i32,
}
