#[derive(Debug, Clone)]
pub enum IData {
	Number(f64),
	Reference(String),
	List(Vec<IData>),
	Struct(std::collections::HashMap<String, IData>),
	Bool(bool),
}
