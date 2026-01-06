#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
	Verify,
	Bvh,
	Obj,
}

impl OutputFormat {
	pub fn to_str(self) -> &'static str {
		match self {
			Self::Verify => "verify",
			Self::Bvh => "bvh",
			Self::Obj => "obj",
		}
	}
}

impl clap::ValueEnum for OutputFormat {
	fn value_variants<'a>() -> &'a [Self] {
		&[Self::Verify, Self::Bvh, Self::Obj]
	}

	fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
		Some(clap::builder::PossibleValue::new(self.to_str()))
	}
}

use std::fmt;
impl fmt::Display for OutputFormat {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.to_str())
	}
}

/// Compile scene yaml files into BVH or OBJ format
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
	/// YAML file path to read scene data from
	#[arg(required = true)]
	pub input: String,

	/// The maximum number of children that a single box node can have. 0 indicates unbounded size.
	#[arg(short = 's', long, default_value_t = 0)]
	pub box_size: u8,

	/// Each box holding multiple nodes is converted into a box holding single-child boxes. In
	/// other words, transforms the scene such that every box either holds one child of any type
	/// OR holds multiple boxes
	#[arg(short, long, action)]
	pub double: bool,

	/// Output format to compile to. Omit to verify scene only.
	#[arg(short, long, default_value_t = OutputFormat::Verify)]
	pub format: OutputFormat,

	/// Verify the output uses no more than the given number of instance levels, fail if not. 0
	/// indicates unbounded. 1 is no instancing. 2 is for two levels: root may use instance. 3
	/// allows an instance to use an instance. Et cetera.
	#[arg(short, long, action, default_value_t = 0)]
	pub instancing: u8,

	/// File to output result to. Omit to output to stdout. Output format will be guessed from the
	/// path's file extension and used unless --format is present.
	#[arg(short, long, default_value_t = String::from(""))]
	pub out: String,

	/// Generate no boxes (cannot be used in generating BVH output!).
	#[arg(short = 'a', long, action)]
	pub raw: bool,

	/// Box the root, even if a single node would suffice.
	#[arg(short, long, action)]
	pub root: bool,

	/// Split tri-strips into individual triangles. Enabled implicitly when generating BVH target
	#[arg(short = 'p', long, action)]
	pub split: bool,

	/// Whether ray and point objects affect dimensions of their containing box
	#[arg(short, long, action, default_value_t = false)]
	pub total_box: bool,

	/// Force instance nodes to hold only boxes directly.
	#[arg(short, long, action)]
	pub wrap: bool,
}
