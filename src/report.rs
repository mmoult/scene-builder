use colored::Colorize;

pub fn warn(msg: &str) {
	println!("{}: {}", "WARN".bold().yellow(), msg);
}

pub fn error(msg: &str) {
	println!("{}: {}", "ERROR".bold().red(), msg);
}
