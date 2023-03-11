use {
	clap::Parser,
	color_eyre::Result as Eyre,
	gokz_rs::{prelude::*, GlobalAPI},
	log::{debug, info},
	serde::{Deserialize, Serialize},
	serde_json::Value as JsonValue,
	std::{
		fs::File,
		io::{self, BufWriter, Write},
		path::{Path, PathBuf},
	},
};

#[derive(Debug, Parser)]
struct Args {
	/// Google Sheets ID for the spreadsheet containing the map release info.
	#[arg(long)]
	sheet: String,

	/// Config file with Steam WebAPI key.
	#[arg(short, long)]
	#[clap(default_value = "./config.toml")]
	config_file: PathBuf,

	/// Output the map data as JSON.
	#[arg(long = "json")]
	json_path: Option<PathBuf>,

	/// Output the map data as SQL migrations.
	#[arg(long = "sql")]
	sql_path: Option<PathBuf>,

	/// Print debug information.
	#[arg(long)]
	#[clap(default_value = "false")]
	debug: bool,
}

#[derive(Debug, Deserialize)]
struct Config {
	google_api_key: String,
}

const URL: &str = "https://sheets.googleapis.com/v4/spreadsheets";

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;
	let args = Args::parse();
	let config_file = std::fs::read_to_string(&args.config_file)?;
	let config: Config = toml::from_str(&config_file)?;
	let client = gokz_rs::Client::new();

	std::env::set_var("RUST_LOG", if args.debug { "fetch_maps=DEBUG" } else { "fetch_maps=INFO" });
	env_logger::init();

	let url = format!("{}/{}?key={}", URL, args.sheet, config.google_api_key);

	let mut data = client
		.get(url)
		.query(&[("includeGridData", true)])
		.send()
		.await?
		.json::<Response>()
		.await?;

	let mut maps = Vec::new();
	for map in data
		.sheets
		.remove(0)
		.data
		.remove(0)
		.row_data
		.into_iter()
		.skip(1)
		.filter_map(|row| {
			let row = row
				.values
				.into_iter()
				.flatten()
				.filter_map(|col| col.formatted_value)
				.collect::<Vec<_>>();

			debug!("Row: {:?}", &row);

			if row.len() < 6 {
				return None;
			}

			Some(Map {
				id: None,
				name: row[0].clone(),
				courses: None,
				tier: row[6].parse().ok()?,
				kzt: row[1] == "Yes",
				skz: row[2] == "Yes",
				vnl: row[3] == "Yes",
				validated: true,
				filesize: row[5].replace(',', "").parse().ok()?,
				created_by: None,
				approved_by: None,
			})
		}) {
		maps.push(
			GlobalAPI::get_map(&MapIdentifier::Name(map.name.clone()), &client)
				.await
				.map(|global_map| Map {
					id: Some(global_map.id as u32),
					..map
				})?,
		);
	}

	debug!("Maps: {:#?}", &maps);
	info!("{} maps", maps.len());

	if let Some(json_path) = args.json_path {
		let json_file = get_file(&json_path)?;
		let mut buf_writer = BufWriter::new(json_file);
		let json = serde_json::to_vec(&maps)?;
		write_to_file(&json, &mut buf_writer, true)?;
		info!("Wrote maps to `{}`.", &json_path.display());
	}

	if let Some(sql_path) = args.sql_path {
		let sql_file = get_file(&sql_path)?;
		let mut buf_writer = BufWriter::new(sql_file);

		/*
		 * +-------------+----------------------+------+-----+---------------------+-------+
		 * | Field       | Type                 | Null | Key | Default             | Extra |
		 * +-------------+----------------------+------+-----+---------------------+-------+
		 * | id          | smallint(5) unsigned | NO   | PRI | NULL                |       |
		 * | name        | varchar(255)         | NO   |     | NULL                |       |
		 * | courses     | tinyint(3) unsigned  | NO   |     | 1                   |       |
		 * | validated   | tinyint(1)           | NO   |     | 0                   |       |
		 * | filesize    | bigint(20) unsigned  | NO   |     | NULL                |       |
		 * | created_by  | int(10) unsigned     | NO   | MUL | NULL                |       |
		 * | approved_by | int(10) unsigned     | NO   | MUL | NULL                |       |
		 * | created_on  | datetime             | NO   |     | current_timestamp() |       |
		 * | updated_on  | datetime             | NO   |     | current_timestamp() |       |
		 * +-------------+----------------------+------+-----+---------------------+-------+
		 */
		let mut sql_maps = String::from(
			r#"
INSERT INTO maps
  (id, name, courses, validated, filesize, created_by, approved_by, created_on, updated_on)
VALUES"#,
		);

		/*
		 * +----------------+----------------------+------+-----+---------+-------+
		 * | Field          | Type                 | Null | Key | Default | Extra |
		 * +----------------+----------------------+------+-----+---------+-------+
		 * | id             | int(10) unsigned     | NO   | PRI | NULL    |       |
		 * | map_id         | smallint(5) unsigned | NO   | MUL | NULL    |       |
		 * | stage          | tinyint(3) unsigned  | NO   |     | NULL    |       |
		 * | kzt            | tinyint(1)           | NO   |     | NULL    |       |
		 * | kzt_difficulty | tinyint(3) unsigned  | NO   |     | NULL    |       |
		 * | skz            | tinyint(1)           | NO   |     | NULL    |       |
		 * | skz_difficulty | tinyint(3) unsigned  | NO   |     | NULL    |       |
		 * | vnl            | tinyint(1)           | NO   |     | NULL    |       |
		 * | vnl_difficulty | tinyint(3) unsigned  | NO   |     | NULL    |       |
		 * +----------------+----------------------+------+-----+---------+-------+
		 */
		let mut sql_courses = String::from(
			r#"
INSERT INTO courses
  (id, map_id, stage, kzt, kzt_difficulty, skz, skz_difficulty, vnl, vnl_difficulty)
VALUES"#,
		);

		for (
			i,
			Map {
				name,
				tier,
				kzt,
				skz,
				vnl,
				validated,
				filesize,
				..
			},
		) in maps.into_iter().enumerate()
		{
			sql_maps.push_str(&format!(
				r#"
  ({i}, "{name}", 0, {validated}, {filesize}, 0, 0, "", ""),"#
			));

			sql_courses.push_str(&format!(
				r#"
  (0, {i}, 0, {kzt}, {tier}, {skz}, {tier}, {vnl}, {tier}),"#
			));
		}

		let trailing_comma = sql_maps.rfind(',').unwrap();
		sql_maps.replace_range(trailing_comma.., ";");
		let trailing_comma = sql_courses.rfind(',').unwrap();
		sql_courses.replace_range(trailing_comma.., ";\n");

		let sql = format!("{sql_maps}\n{sql_courses}");

		write_to_file(sql.as_bytes(), &mut buf_writer, true)?;
		info!("Wrote maps to `{}`.", &sql_path.display());
	}

	info!("Spreadsheet: {}", data.spreadsheet_url);

	Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Map {
	id: Option<u32>,
	name: String,
	courses: Option<u8>,
	tier: u8,
	kzt: bool,
	skz: bool,
	vnl: bool,
	validated: bool,
	filesize: u64,
	created_by: Option<u32>,
	approved_by: Option<u32>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
	spreadsheet_id: String,
	properties: JsonValue,
	sheets: Vec<Sheet>,
	spreadsheet_url: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Sheet {
	properties: JsonValue,
	data: Vec<SheetData>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SheetData {
	row_data: Vec<Row>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Row {
	values: Option<Vec<RowValue>>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RowValue {
	user_entered_value: Option<JsonValue>,
	effective_value: Option<JsonValue>,
	formatted_value: Option<String>,
	user_entered_format: Option<JsonValue>,
	effective_format: Option<JsonValue>,
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

fn get_file(path: &Path) -> io::Result<File> {
	match File::options().write(true).open(path) {
		Ok(file) => Ok(file),
		Err(why) => {
			if matches!(why.kind(), io::ErrorKind::NotFound) {
				File::create(path)
			} else {
				Err(why)
			}
		}
	}
}
