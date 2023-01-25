use {
	color_eyre::{eyre::eyre, Result as Eyre},
	log::info,
	std::{
		fs::File,
		io::{BufWriter, ErrorKind::NotFound, Write},
	},
};

pub(crate) fn get_file(path: &str) -> Eyre<File> {
	match File::options().write(true).open(path) {
		Ok(file) => Ok(file),
		Err(why) => match why.kind() {
			NotFound => Ok(File::create(path)?),
			_ => Err(eyre!(why)),
		},
	}
}

pub(crate) fn write_to_file<W: Write>(
	buf_writer: &mut BufWriter<W>,
	bytes: &[u8],
	path: &str,
) -> Eyre<()> {
	buf_writer.write_all(bytes)?;
	buf_writer.flush()?;
	info!("Wrote `{}` bytes to `{}`.", bytes.len(), path);
	Ok(())
}
