mod up;
pub use up::up;

mod down;
pub use down::down;

pub mod schemas;
pub mod util;

use clap::{Subcommand, ValueEnum};

#[derive(Debug, Clone, Subcommand)]
pub enum SqlAction {
	Up,
	Down,
	Redo,
	Insert { schema: Schema, data: String },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Schema {
	Players,
	Modes,
	Servers,
	Maps,
	Courses,
	Records,
	Mappers,
}

pub fn sanitize(input: &str) -> String {
	input.replace(['\'', '"', ',', '\\'], "")
}
