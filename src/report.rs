use colored::Colorize;

pub fn warn(msg: &str) {
	eprintln!("{}: {}", "WARN".bold().yellow(), msg);
}
