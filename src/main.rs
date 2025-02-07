mod parse;
mod types;

#[derive(Debug, Clone, Copy, PartialEq)]
enum OutputFormat {
	Verify,
	Json, // BVH
	Obj,
}

impl OutputFormat {
	fn to_str(&self) -> &'static str {
		match self {
			Self::Verify => "verify",
			Self::Json => "json",
			Self::Obj => "obj",
		}
	}
}

impl clap::ValueEnum for OutputFormat {
	fn value_variants<'a>() -> &'a [Self] { &[Self::Verify, Self::Json, Self::Obj] }

	fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
		Some(clap::builder::PossibleValue::new(self.to_str()))
	}
}

use std::fmt;
impl fmt::Display for OutputFormat {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.to_str()) }
}

/// Compile scene yaml files into BVH or OBJ format
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// The maximum number of children that a single box node can have. 0 indicates unbounded size.
	#[arg(short, long, default_value_t = 0)]
	box_size: u8,

	/// Each box holding multiple nodes is converted into a box holding single-child boxes
	#[arg(short, long, action)]
	double: bool,

	/// Output format to compile to. Omit to verify scene only.
	#[arg(short, long, default_value_t = OutputFormat::Verify)]
	format: OutputFormat,

	/// File to output result to. Omit to output to stdout. Output format will be guessed from the
	/// path's file extension and used unless --format is present.
	#[arg(short, long, default_value_t = String::from(""))]
	out: String,

	/// Generate no boxes (cannot be used in generating BVH output!).
	#[arg(short, long, action)]
	raw: bool,

	/// Box the root, even if a single node would suffice.
	#[arg(short = 't', long, action)]
	root: bool,

	/// Force instance nodes to hold only boxes.
	#[arg(short, long, action)]
	wrap: bool,

	/// YAML file path to read scene data from
	#[arg(required = true)]
	input: String,
}

fn main() -> Result<(), String> {
	use clap::Parser;
	let args = Args::parse();

	let out_format = if args.format != OutputFormat::Verify {
		args.format
	} else if args.out.ends_with(".json") {
		OutputFormat::Json
	} else if args.out.ends_with(".obj") {
		OutputFormat::Obj
	} else {
		return Err(String::from("Cannot deduce output type!"));
	};

	if !(args.format == OutputFormat::Verify || args.format == OutputFormat::Obj) && args.raw {
		return Err(String::from(
			"Cannot use command line option 'raw' when outputting BVH data!",
		));
	}

	// parse file and check syntax
	let input = match parse::parse_file(&args.input) {
		Ok(dat) => dat,
		Err(msg) => return Err(msg),
	};

	// Convert from input data to IR data by checking grammar
	// todo!("convert from input data to IR data by checking grammar");

	match out_format {
		OutputFormat::Verify => {},
		OutputFormat::Json => {},
		OutputFormat::Obj => {},
	}

	Ok(())
}
