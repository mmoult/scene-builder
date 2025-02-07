use super::types::IData;
use std::iter::Peekable;

fn count_indent<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> u32 {
	let mut indent = 0;
	let mut in_comment = false;
	loop {
		let c = match chars.peek() {
			None => return 0,
			Some(c) => *c,
		};

		if in_comment {
			if c == '\n' {
				in_comment = false;
			}
		} else if c == '#' {
			// comment until end of the line. Count indent on next
			in_comment = true;
			indent = 0;
		} else if c == '\n' {
			indent = 0; // blank line - reset for next
		} else if c == ' ' {
			// YAML only allows spaces for indents
			indent += 1;
		} else {
			// Semantically relevant character. Indent ends here
			break;
		}

		// We accept this progress (the character seen doesn't need to be processed again), so apply
		// the peeked move
		chars.next();
	}

	return indent;
}

use std::collections::HashMap;

fn new_entry(map: &mut HashMap<String, IData>, key: String, val: IData) -> Result<(), String> {
	use std::collections::hash_map::Entry;
	return match map.entry(key.clone()) {
		Entry::Occupied(_) => Err(format!(
			"Cannot add variable by name {key} when one already exists!"
		)),
		Entry::Vacant(v) => {
			v.insert(val);
			Ok(())
		},
	};
}

fn parse_variable<I: Iterator<Item = char>>(
	chars: &mut Peekable<I>,
	min_indent: u32,
) -> Result<(String, IData), String> {
	return Err(String::from("bad"));
}

pub fn parse_file(path: &str) -> Result<IData, String> {
	// Load the scene file
	let file = match std::fs::read_to_string(path) {
		Ok(got_text) => got_text,
		Err(_) => {
			let mut err = String::from("Could not read input file: \"");
			err.push_str(path);
			err.push_str("\"!");
			return Err(err);
		},
	};

	let mut chars = file.chars().peekable();
	let mut fields: HashMap<String, IData> = HashMap::new();

	loop {
		let indent = count_indent(&mut chars);
		if indent > 0 {
			return Err(format!("Variable at file root defined at indent {indent}!"));
		}
		match chars.peek() {
			None => break,
			Some(_) => {},
		}

		match parse_variable(&mut chars, 0) {
			Err(e) => return Err(e),
			Ok((key, val)) => match new_entry(&mut fields, key, val) {
				Err(e) => return Err(e),
				Ok(_) => {},
			},
		};
	}
	// Empty file is permissible.

	// Verify that nothing comes after the mapping
	// TODO

	Ok(IData::Struct(fields))
}
