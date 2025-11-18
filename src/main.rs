mod bvh;
mod ir;
mod obj;
mod report;
mod transform;

#[derive(Debug, Clone, Copy, PartialEq)]
enum OutputFormat {
	Verify,
	Bvh,
	Obj,
}

impl OutputFormat {
	fn to_str(self) -> &'static str {
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
struct Args {
	/// The maximum number of children that a single box node can have. 0 indicates unbounded size.
	#[arg(short, long, default_value_t = 0)]
	box_size: u8,

	/// Each box holding multiple nodes is converted into a box holding single-child boxes. In
	/// other words, transforms the scene such that every box either holds one child of any type
	/// OR holds multiple boxes
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

	/// Split tri-strips into individual triangles. Enabled implicitly when generating BVH target
	#[arg(short = 'p', long, action)]
	split: bool,

	/// Force instance nodes to hold only boxes directly.
	#[arg(short, long, action)]
	wrap: bool,

	/// YAML file path to read scene data from
	#[arg(required = true)]
	input: String,
}

fn main() -> Result<(), String> {
	use clap::Parser;
	let args = Args::parse();

	let out_format = if args.format != OutputFormat::Verify || args.out.is_empty() {
		args.format
	} else if args.out.ends_with(".json")
		|| args.out.ends_with(".yaml")
		|| args.out.ends_with(".yml")
	{
		OutputFormat::Bvh
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
	let path = args.input;
	let file = match std::fs::read_to_string(&path) {
		Ok(got_text) => got_text,
		Err(_) => return Err(format!("Could not read input file: \"{path}\"!")),
	};
	use yaml_rust2::YamlLoader;
	let docs = match YamlLoader::load_from_str(file.as_str()) {
		Ok(docs) => docs,
		Err(_) => return Err("Could not parse YAML from given file!".to_string()),
	};

	let num_docs = docs.len();
	if num_docs != 1 {
		return Err(format!(
			"Incompatible number of YAML documents found in input! 1 expected, but {num_docs} \
			 seen."
		));
	}

	// Convert from input data to IR data by checking grammar
	let mut scene = ir::to_ir(&docs[0])?;

	// If we are simply verifying the scene, we are done now.
	if let OutputFormat::Verify = out_format {
		if !args.out.is_empty() {
			return Err(format!(
				"Cannot print to \"{}\" because verification mode is enabled!",
				args.out
			));
		}
		return Ok(());
	}
	// Otherwise, we want to apply transformations given by the command line arguments. Then we can
	// translate into the target format.
	if args.raw {
		if let OutputFormat::Bvh = out_format {
			return Err("Cannot use option `raw` with a BVH target!".to_string());
		}
	} else {
		// Handle all the box-related transformations
		transform::transform(
			&mut scene,
			args.root,
			args.wrap,
			args.box_size,
			args.double,
			out_format == OutputFormat::Bvh || args.split,
		);
	}

	let lines = match out_format {
		OutputFormat::Bvh => bvh::to_bvh(&scene),
		OutputFormat::Obj => obj::to_obj(&scene),
		OutputFormat::Verify => panic!("Verify case should have exited earlier!"),
	};
	if args.out.is_empty() {
		for line in lines.iter() {
			println!("{}", line);
		}
	} else {
		use std::fs::File;
		let mut writer = match File::create(&args.out) {
			Ok(f) => f,
			Err(_) => return Err(format!("Could not write output to file \"{}\"!", &args.out)),
		};
		use std::io::Write;
		for line in lines.iter() {
			match writeln!(writer, "{}", line) {
				Ok(_) => {},
				Err(_) => {
					return Err(format!(
						"Failure in writing output to file \"{}\"!",
						&args.out
					));
				},
			}
		}
	}

	Ok(())
}
