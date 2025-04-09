use std::collections::HashMap;

#[derive(Copy, Clone)]
pub enum Node {
	// literal values
	Number(f64),
	Bool(bool),
	// link to some other value held by the scene
	Strip(usize),
	Ray(usize),
	Instance(usize),
	Mapping(usize),
	Sequence(usize),
}

use std::fmt;
impl fmt::Display for Node {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Node::Number(v) => write!(f, "{}", v),
			Node::Bool(v) => write!(f, "{}", v),
			Node::Strip(i) => write!(f, "Strip{}", i),
			Node::Ray(i) => write!(f, "Ray{}", i),
			Node::Instance(i) => write!(f, "Instance{}", i),
			Node::Mapping(i) => write!(f, "Mapping{}", i),
			Node::Sequence(i) => write!(f, "Sequence{}", i),
		}
	}
}

pub type Point3D = nalgebra::Vector3<f64>;
fn new_point(val: f64) -> Point3D { Point3D::new(val, val, val) }

pub struct Strip {
	pub vals: Vec<Point3D>,
}
impl Strip {
	fn new() -> Strip { Strip { vals: vec![] } }
}

pub struct Ray {
	pub origin: Point3D,
	pub direction: Point3D,
	pub extent: f64,
	pub fields: HashMap<String, Node>,
}

pub struct Instance {
	pub affected: Node,
	pub scale: Point3D,
	pub rotate: Point3D,
	pub translate: Point3D,
	pub fields: HashMap<String, Node>,
}

pub struct Mapping {
	pub fields: HashMap<String, Node>,
}
impl Mapping {
	fn new() -> Mapping {
		Mapping {
			fields: HashMap::new(),
		}
	}
}

pub struct Sequence {
	pub vals: Vec<Node>,
}
impl Sequence {
	fn new() -> Sequence { Sequence { vals: vec![] } }
}

pub struct Scene {
	pub world: Node,
	pub strips: Vec<Strip>,
	pub rays: Vec<Ray>,
	pub instances: Vec<Instance>,
	pub mappings: Vec<Mapping>,
	pub sequences: Vec<Sequence>,
}

fn as_3d(scene: &Scene, node: &Node) -> Result<Point3D, String> {
	match node {
		Node::Sequence(seq_at) => {
			// Not only must this be a sequence, but it must have three elements, each of
			// which must resolve to a number
			let seq = &scene.sequences[*seq_at];
			let len = seq.vals.len();
			if len != 3 {
				return Err(format!(
					"Could not resolve 3D point from a sequence with {len} dimensions!"
				));
			}
			let mut ret = new_point(0.0);
			for i in 0..3 {
				match seq.vals[i] {
					Node::Number(num) => ret[i] = num,
					_ => {
						return Err(format!(
							"Could not resolve numeric component of 3D point from {}!",
							seq.vals[i]
						));
					},
				}
			}

			Ok(ret)
		},
		_ => Err(format!("Could not resolve 3D point from {}", node)),
	}
}

fn resolve<'a>(namespace: &[usize], scene: &'a Scene, name: &str) -> Option<&'a Node> {
	for idx in namespace.iter().rev() {
		match scene.mappings[*idx].fields.get(name) {
			None => {},
			Some(found) => return Some(found),
		}
	}
	None
}

fn parse(input: &Yaml, namespace: &mut Vec<usize>, scene: &mut Scene) -> Result<Node, String> {
	let ret = match input {
		Yaml::Real(fp) => match fp.parse::<f64>() {
			Ok(val) => Node::Number(val),
			Err(_) => return Err(format!("Could not parse float number {fp}!")),
		},
		Yaml::Integer(val) => Node::Number(*val as f64),
		Yaml::String(name) => match resolve(namespace, scene, name) {
			Some(found) => *found,
			None => return Err(format!("Could not resolve reference \"{name}\"!")),
		},
		Yaml::Boolean(val) => Node::Bool(*val),
		Yaml::Array(arr) => {
			let seq_at = scene.sequences.len();
			scene.sequences.push(Sequence::new());

			for element in arr {
				let node = parse(element, namespace, scene)?;
				scene.sequences[seq_at].vals.push(node);
			}

			Node::Sequence(seq_at)
		},
		Yaml::Hash(map) => {
			let name_at = scene.mappings.len();
			scene.mappings.push(Mapping::new());
			namespace.push(name_at);
			for (name, val) in map.iter() {
				let name = match name {
					Yaml::String(n) => n,
					_ => return Err("Name in YAML field found to be non-string!".to_string()),
				};
				let node = parse(val, namespace, scene)?;
				scene.mappings[name_at].fields.insert(name.clone(), node);
			}
			let mut ret = Node::Mapping(name_at);

			// Create the result from the top namespace. Recognize various types:
			if let Some(node) = scene.mappings[name_at].fields.get("data") {
				// Check that data is actually a sequence. That is all we require of it
				match node {
					Node::Sequence(_) => (),
					_ => {
						return Err("Field `data` must be a sequence!".to_string());
					},
				}
			} else if scene.mappings[name_at].fields.contains_key("instance") {
				// This is not, in fact, a custom, it is an instance. Convert it to such
				let mut affected = Node::Bool(false); // guaranteed to be replaced since conditional forces it
				// These can be replaced, but all are optional:
				let mut scale = new_point(1.0);
				let mut rotate = new_point(0.0);
				let mut translate = new_point(0.0);
				let mut fields = HashMap::new();

				for (key, value) in scene.mappings[name_at].fields.iter() {
					if key == "instance" {
						affected = *value;
					} else if key == "scale" {
						scale = as_3d(scene, value)?;
					} else if key == "rotate" {
						rotate = as_3d(scene, value)?;
					} else if key == "translate" {
						translate = as_3d(scene, value)?;
					} else {
						fields.insert(key.clone(), *value);
					}
				}
				let inst = Instance {
					affected,
					scale,
					rotate,
					translate,
					fields,
				};
				let scene_at = scene.instances.len();
				scene.instances.push(inst);
				// We can safely remove the old mapping since we parsed it directly (and therefore,
				// it couldn't have saved and referenced elsewhere).
				scene.mappings.pop();
				ret = Node::Instance(scene_at);
			}
			if scene.mappings[name_at].fields.contains_key("strip") {
				match scene.mappings[name_at].fields["strip"] {
					Node::Sequence(idx) => {
						// Attempt to convert the sequence at the index into a strip
						// A strip is a list of 3D points, so we convert each and add them to a
						// running strip object
						let mut strip = Strip::new();
						for element in scene.sequences[idx].vals.iter() {
							strip.vals.push(as_3d(scene, element)?);
						}
						let vertices = strip.vals.len();
						if vertices < 3 {
							return Err(format!(
								"Cannot create a strip with only {vertices} vertices! Must have \
								 at least 3."
							));
						}
						let strip_at = scene.strips.len();
						scene.strips.push(strip);
						// Replace the pre-existing sequence with the created strip in-place
						scene.mappings[name_at]
							.fields
							.insert("strip".to_string(), Node::Strip(strip_at));
					},
					_ => {
						return Err("Unexpected value found for `strip` keyword. Must be a \
						            sequence!"
							.to_string());
					},
				}
			}
			if scene.mappings[name_at].fields.contains_key("ray") {
				match scene.mappings[name_at].fields["ray"] {
					Node::Mapping(idx) => {
						// Attempt to convert mapping into a ray
						let mut origin = None;
						let mut direction = None;
						let mut extent = None;
						let mut fields = HashMap::new();

						for (key, value) in scene.mappings[idx].fields.iter() {
							if key == "origin" {
								origin = Some(as_3d(scene, value)?);
							} else if key == "direction" {
								direction = Some(as_3d(scene, value)?);
							} else if key == "extent" {
								extent = match value {
									Node::Number(num) => Some(*num),
									_ => {
										return Err(format!(
											"Expected number for `ray` field `extent`, but \
											 {value} was found instead!"
										));
									},
								};
							} else {
								fields.insert(key.clone(), *value);
							}
						}
						let origin = match origin {
							None => {
								return Err("Missing field `origin` in ray object!".to_string());
							},
							Some(u) => u,
						};
						let direction = match direction {
							None => {
								return Err("Missing field `direction` in ray object!".to_string());
							},
							Some(u) => u,
						};
						let extent = match extent {
							None => {
								return Err("Missing field `extent` in ray object!".to_string());
							},
							Some(u) => u,
						};
						let ray = Ray {
							origin,
							direction,
							extent,
							fields,
						};
						let ray_at = scene.rays.len();
						scene.rays.push(ray);
						scene.mappings[name_at]
							.fields
							.insert("ray".to_string(), Node::Ray(ray_at));
					},
					_ => {
						return Err(
							"Unexpected value found for `ray` keyword. Must be a mapping!"
								.to_string(),
						);
					},
				}
			}

			namespace.pop();
			ret
		},
		_ => return Err("Unsupported YAML value found while parsing scene data!".to_string()),
	};
	Ok(ret)
}

use yaml_rust2::Yaml;
pub fn to_ir(input: &Yaml) -> Result<Scene, String> {
	let mut scene = Scene {
		world: Node::Bool(false),
		strips: vec![],
		rays: vec![],
		instances: vec![],
		mappings: vec![],
		sequences: vec![],
	};

	let mut namespace: Vec<usize> = vec![];
	scene.world = parse(input, &mut namespace, &mut scene)?;

	Ok(scene)
}
