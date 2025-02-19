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

	indent
}

use std::collections::HashMap;

fn new_entry(map: &mut HashMap<String, IData>, key: String, val: IData) -> Result<(), String> {
	use std::collections::hash_map::Entry;
	match map.entry(key.clone()) {
		Entry::Occupied(_) => Err(format!(
			"Cannot add variable by name {key} when one already exists!"
		)),
		Entry::Vacant(v) => {
			v.insert(val);
			Ok(())
		},
	}
}

fn parse_string<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> (String, bool) {
	// Strings may use '' for literal strings or "" for escape sequences. If a string utilizes
	// quotes, the quotes must cover the entire string, i.e. the first and last character in the
	// string must be the quotes.
	let mut value: String = String::from("");

	#[derive(PartialEq)]
	enum StringStatus {
		None,
		Double,
		Single,
	}
	let mut in_str = StringStatus::None;

	let mut first = true;
	let mut escape = false;

	loop {
		let c = match chars.peek() {
			None => break,
			Some(c) => *c,
		};

		if in_str != StringStatus::None {
			if in_str == StringStatus::Double {
				if c == '\\' {
					escape = !escape;
					chars.next();
					continue;
				} else if c == '"' && !escape {
					chars.next();
					break; // string done after closing quotes
				}
				escape = false; // reset escape, which is only used in double quote strings
			} else if in_str == StringStatus::Single && c == '\'' {
				chars.next();
				break; // again, string done after closing quote
			}
			value.push(c);
		} else if c == '\n' || c == '#' || c == ':' {
			break; // string has ended
		} else {
			if first {
				first = false;
				if c == '"' || c == '\'' {
					in_str = if c == '"' {
						StringStatus::Double
					} else {
						StringStatus::Single
					};
					chars.next();
					continue;
				}
			}
			value.push(c);
		}

		chars.next();
	}
	(value, in_str == StringStatus::None)
}

/// @brief Skips whitespace
/// @param chars the characters to read from
/// @param break_newline whether to stop at newlines (true) or treat them as regular space (false)
/// @return next non-whitespace or newline if valid
fn skip_whitespace<I: Iterator<Item = char>>(
	chars: &mut Peekable<I>,
	break_newline: bool,
) -> Option<char> {
	loop {
		let c = match chars.peek() {
			None => return None,
			Some(c) => *c,
		};

		if c == '#' {
			// comment until end of line
			loop {
				let c = match chars.peek() {
					None => return None,
					Some(c) => *c,
				};
				if c == '\n' {
					break;
				}
			}
			if break_newline {
				return Some(c);
			}
		} else if !c.is_whitespace() || (break_newline && c == '\n') {
			return Some(c); // semantically relevant character
		}

		chars.next();
	}
}

fn verify_blank<I: Iterator<Item = char>>(
	chars: &mut Peekable<I>,
	break_at_newline: bool,
) -> Result<(), String> {
	loop {
		let c = match skip_whitespace(chars, break_at_newline) {
			None => break,
			Some(ch) => ch,
		};
		if c == '\n' {
			// Should only be triggered if break at newline true
			break;
		} else if !c.is_whitespace() {
			return Err(format!("Unexpected character ({c}) found after value!"));
		}
	}
	Ok(())
}

fn parse_number<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> Result<f64, String> {
	// Create a string from the iterator which contains the whole number, then use the std function
	// to parse out the float from the string fetched
	let mut build: String = String::new();
	let mut first = true;
	let mut seen_dec = false;
	loop {
		let c = match chars.peek() {
			None => break,
			Some(ch) => *ch,
		};
		let mut ok = false;
		if first {
			first = false;
			if c == '-' {
				ok = true;
			}
		}
		if c == '.' && !seen_dec {
			seen_dec = true;
			ok = true;
		}
		if c.is_ascii_digit() {
			ok = true;
		}

		if !ok {
			break;
		}
		build.push(c);
		chars.next();
	}
	use std::str::FromStr;
	match f64::from_str(&build) {
		Ok(num) => Ok(num),
		Err(err) => Err(format!("{err}")),
	}
}

fn build_mapping(names: Vec<String>, fields: Vec<IData>) -> Result<IData, String> {
	let mut map = HashMap::new();
	for (key, val) in names.iter().zip(fields.iter()) {
		new_entry(&mut map, key.clone(), val.clone())?;
	}
	Ok(IData::Struct(map))
}

fn parse_inline_agg<I: Iterator<Item = char>>(
	chars: &mut Peekable<I>,
	is_sequence: bool,
) -> Result<IData, String> {
	// skip over the [ or {, which has already been seen
	chars.next();

	let mut elements = vec![];
	let mut names = vec![];

	loop {
		let c = match skip_whitespace(chars, false) {
			None => return Err(String::from("Premature end found while parsing aggregate!")),
			Some(ch) => ch,
		};

		if (is_sequence && c == ']') || (!is_sequence && c == '}') {
			// Consume the end token
			chars.next();
			break;
		}

		// Parse out an element
		if is_sequence {
			let (data, _) = parse_value(chars, 0)?;
			elements.push(data);
			// We don't expect inline lists to rollover, but if they do, we don't care about it so
			// long as we see a concluding ]
		} else {
			let (key, val) = parse_variable(chars, 0, false)?;
			names.push(key);
			elements.push(val);
		}

		// Allow comma after each element ((even after final element))
		match skip_whitespace(chars, false) {
			None => (),
			Some(c) =>
				if c == ',' {
					chars.next();
				} else if (is_sequence && c != ']') || (!is_sequence && c != '}') {
					return Err(String::from(
						"Missing comma between elements in inline aggregate!",
					));
				},
		}
	}
	// Now that we are done parsing, add elements and form the type:
	Ok(if is_sequence {
		IData::List(elements)
	} else {
		build_mapping(names, elements)?
	})
}

fn parse_agg<I: Iterator<Item = char>>(
	chars: &mut Peekable<I>,
	indent: u32,
	is_sequence: bool,
) -> Result<(IData, bool), String> {
	let mut elements = vec![];
	let mut names = vec![];
	// We have already seen the indent at the start of the first element
	// The indent has been saved as indent
	loop {
		if is_sequence {
			// Must see '-' and then some optional space
			let ch = match chars.peek() {
				None => '\0',
				Some(ch) => *ch,
			};
			if ch != '-' {
				// The list is done because this line doesn't have a bullet. This cannot happen on
				// the first element because we must see a bullet to get to this logic.
				break;
			}
			chars.next();
			// We should know validity because we checked when first identifying or asserting
			// indentation
			let (element, new_line) = parse_value(chars, indent)?;
			if !new_line {
				verify_blank(chars, true)?;
			}
			elements.push(element);
		} else {
			let (key, val) = parse_variable(chars, 0, false)?;
			names.push(key);
			elements.push(val);
		}

		// parseVariable or verifyBlank have taken the courtesy of going to the next line for us.
		// We want to see if the next line has the correct indent or if it is out of this aggregate
		let next = count_indent(chars);
		// next == 0 if we reached end of file
		use std::cmp::Ordering;
		match next.cmp(&indent) {
			Ordering::Less => break,
			Ordering::Equal => {},
			Ordering::Greater => {
				// We cannot suddenly get a block with a larger indent
				return Err(format!(
					"Encountered block while parsing aggregate with indent {next} where {indent} \
					 was expected!"
				));
			},
		}
	}
	// Reset to the start of the line so the next to process has the correct indent count
	// TODO: Need to work out how to do this!
	todo!("Need to work out how to reset chars to line start!");
	// Now that we are done parsing, add elements and form the type:
	Ok((
		if is_sequence {
			IData::List(elements)
		} else {
			build_mapping(names, elements)?
		},
		true,
	))
}

fn parse_value<I: Iterator<Item = char>>(
	chars: &mut Peekable<I>,
	min_indent: u32,
) -> Result<(IData, bool), String> {
	match skip_whitespace(chars, true) {
		None => (),
		Some(c) => {
			// Inline sequences or mappings
			if c == '[' || c == '{' {
				let data = parse_inline_agg(chars, c == '[')?;
				return Ok((data, false));
			} else if c == '\n' {
				// Nothing on this line, so it must be an aggregate
				let next = count_indent(chars);
				if next < min_indent {
					return Err(format!(
						"{next} indents seen in block expecting at least {min_indent}!"
					));
				}
				// If we see a -, then this is a list. Otherwise, it is a map
				let c = match chars.peek() {
					None => '\0',
					Some(ch) => *ch,
				};
				return parse_agg(chars, next, c == '-');
				// intentional fallthrough after None to error later
			} else if c == '-' || c == '.' || c.is_ascii_digit() {
				return Ok((IData::Number(parse_number(chars)?), false));
			} else {
				let (str, typical) = parse_string(chars);
				// Note: true, false are forbidden field names- they are instead handled as booleans
				if typical && (str == "true" || str == "false") {
					return Ok((IData::Bool(str == "true"), false));
				}
				return Ok((IData::Reference(str), false));
			}
		},
	}
	Err(String::from("No value can be found!"))
}

fn parse_variable<I: Iterator<Item = char>>(
	chars: &mut Peekable<I>,
	min_indent: u32,
	end_check: bool,
) -> Result<(String, IData), String> {
	let (key, _) = parse_string(chars);
	if skip_whitespace(chars, true).unwrap_or('\0') != ':' {
		return Err(format!("Missing colon in definition for \"{key}\"!"));
	}
	chars.next(); // assuming the next character was a colon, skip over it / consume it and continue

	let (val, next_line) = parse_value(chars, min_indent)?;

	// queue up the next line (and verify there is no more content on this)
	if !next_line && end_check {
		verify_blank(chars, true)?;
	}
	Ok((key, val))
}

pub fn parse_file(path: &str) -> Result<IData, String> {
	// Load the scene file
	let file = match std::fs::read_to_string(path) {
		Ok(got_text) => got_text,
		Err(_) => return Err(format!("Could not read input file: \"{path}\"!")),
	};

	let mut chars = file.chars().peekable();
	let mut fields: HashMap<String, IData> = HashMap::new();

	loop {
		let indent = count_indent(&mut chars);
		if indent > 0 {
			return Err(format!("Variable at file root defined at indent {indent}!"));
		}
		if chars.peek().is_none() {
			break;
		}

		let (key, val) = parse_variable(&mut chars, 0, false)?;
		new_entry(&mut fields, key, val)?;
	}
	// Empty file is permissible.

	// Verify that nothing comes after the mapping
	// TODO

	Ok(IData::Struct(fields))
}
