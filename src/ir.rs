use std::collections::HashMap;

#[derive(Copy, Clone)]
pub enum Node {
	// literal values
	Number(f64),
	Bool(bool),
	// link to some other value held by the scene
	Sequence(usize),
	Strip(usize),
	Ray(usize),
	Instance(usize),
	Mapping(usize),
}

use std::fmt;
impl fmt::Display for Node {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Node::Number(v) => write!(f, "{}", v),
			Node::Bool(v) => write!(f, "{}", v),
			Node::Sequence(i) => write!(f, "Sequence{}", i),
			Node::Strip(i) => write!(f, "Strip{}", i),
			Node::Ray(i) => write!(f, "Ray{}", i),
			Node::Instance(i) => write!(f, "Instance{}", i),
			Node::Mapping(i) => write!(f, "Mapping{}", i),
		}
	}
}

pub struct Sequence {
	pub vals: Vec<Node>,
}
impl Sequence {
	fn new() -> Sequence { Sequence { vals: vec![] } }
}

pub type Point3D = nalgebra::Vector3<f64>;
pub fn new_point(val: f64) -> Point3D { Point3D::new(val, val, val) }

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

pub type TransformMat = nalgebra::Matrix4x3<f64>;
pub type SquareMat = nalgebra::Matrix4<f64>;
pub type HomoPoint = nalgebra::Vector4<f64>;

pub struct Instance {
	pub affected: Node,
	pub scale: Point3D,
	pub rotate: Point3D,
	pub translate: Point3D,
	pub fields: HashMap<String, Node>,
}
impl Instance {
	pub fn obj_to_world(&self) -> TransformMat {
		let scale_mat = matrix![
			self.scale.x, 0.0, 0.0;
			0.0, self.scale.y, 0.0;
			0.0, 0.0, self.scale.z;
		];
		let rotate_rad = Point3D::new(
			self.rotate.x.to_radians(),
			self.rotate.y.to_radians(),
			self.rotate.z.to_radians(),
		);
		let rx = matrix![
			1.0, 0.0, 0.0;
			0.0, rotate_rad.x.cos(), rotate_rad.x.sin();
			0.0, -rotate_rad.x.sin(), rotate_rad.x.cos();
		];
		let ry = matrix![
			rotate_rad.y.cos(), 0.0, -rotate_rad.y.sin();
			0.0, 1.0, 0.0;
			rotate_rad.y.sin(), 0.0, rotate_rad.x.cos();
		];
		let rz = matrix![
			rotate_rad.y.cos(), rotate_rad.y.sin(), 0.0;
			-rotate_rad.y.sin(), rotate_rad.x.cos(), 0.0;
			0.0, 0.0, 1.0;
		];

		let m = scale_mat * rx * ry * rz;
		// contruct a homogenous matrix to allow for translation
		matrix![
			m[(0, 0)], m[(0, 1)], m[(0, 2)];
			m[(1, 0)], m[(1, 1)], m[(1, 2)];
			m[(2, 0)], m[(2, 1)], m[(2, 2)];
			self.translate.x, self.translate.y, self.translate.z;
		]
	}

	#[allow(unused)]
	pub fn world_to_obj(&self) -> TransformMat {
		let scale_mat = matrix![
			1.0 / self.scale.x, 0.0, 0.0;
			0.0, 1.0 / self.scale.y, 0.0;
			0.0, 0.0, 1.0 / self.scale.z;
		];
		let rotate_rad = Point3D::new(
			-self.rotate.x.to_radians(),
			-self.rotate.y.to_radians(),
			-self.rotate.z.to_radians(),
		);
		let rx = matrix![
			1.0, 0.0, 0.0;
			0.0, rotate_rad.x.cos(), rotate_rad.x.sin();
			0.0, -rotate_rad.x.sin(), rotate_rad.x.cos();
		];
		let ry = matrix![
			rotate_rad.y.cos(), 0.0, -rotate_rad.y.sin();
			0.0, 1.0, 0.0;
			rotate_rad.y.sin(), 0.0, rotate_rad.x.cos();
		];
		let rz = matrix![
			rotate_rad.y.cos(), rotate_rad.y.sin(), 0.0;
			-rotate_rad.y.sin(), rotate_rad.x.cos(), 0.0;
			0.0, 0.0, 1.0;
		];

		let m = rz * ry * rx * scale_mat;
		// contruct a homogenous matrix to allow for translation
		matrix![
			m[(0, 0)], m[(0, 1)], m[(0, 2)];
			m[(1, 0)], m[(1, 1)], m[(1, 2)];
			m[(2, 0)], m[(2, 1)], m[(2, 2)];
			self.translate.x, self.translate.y, self.translate.z;
		]
	}

	pub fn homogenize(&self, m: &TransformMat) -> SquareMat {
		matrix![
			m[(0, 0)], m[(0, 1)], m[(0, 2)], 0.0;
			m[(1, 0)], m[(1, 1)], m[(1, 2)], 0.0;
			m[(2, 0)], m[(2, 1)], m[(2, 2)], 0.0;
			m[(3, 0)], m[(3, 1)], m[(3, 2)], 1.0;
		]
	}

	pub fn heterogenize(&self, v: &HomoPoint) -> Point3D { Point3D::new(v.x, v.y, v.z) }
}

pub struct Mapping {
	pub fields: HashMap<String, Node>,
	pub is_box: bool,
	pub min: Point3D,
	pub max: Point3D,
}
impl Mapping {
	fn new() -> Mapping {
		Mapping {
			fields: HashMap::new(),
			is_box: false,
			min: new_point(0.0),
			max: new_point(0.0),
		}
	}

	pub fn as_box(&mut self, min: &Point3D, max: &Point3D) {
		self.is_box = true;
		self.min = *min;
		self.max = *max;
	}
}

pub struct Scene {
	pub world: Node,
	pub sequences: Vec<Sequence>,
	pub strips: Vec<Strip>,
	pub rays: Vec<Ray>,
	pub instances: Vec<Instance>,
	pub mappings: Vec<Mapping>,
}

pub fn as_3d(scene: &Scene, node: &Node) -> Result<Point3D, String> {
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
			// Aggressively try to turn this sequence into a strip. Even if it isn't intended to be
			// a strip, but it matches all the characteristics, we don't lose anything by
			// normalizing it.
			let mut nodes = vec![];
			let mut points = vec![];
			let mut can_strip = arr.len() >= 3;

			for element in arr {
				let node = parse(element, namespace, scene)?;
				nodes.push(node);
				if can_strip {
					match as_3d(scene, &node) {
						Ok(point) => {
							points.push(point);
						},
						Err(_) => {
							can_strip = false;
						},
					}
				}
			}

			if can_strip {
				let strip_at = scene.strips.len();
				scene.strips.push(Strip::new());
				for point in points {
					scene.strips[strip_at].vals.push(point);
				}
				Node::Strip(strip_at)
			} else {
				let seq_at = scene.sequences.len();
				scene.sequences.push(Sequence::new());
				for node in nodes {
					scene.sequences[seq_at].vals.push(node);
				}
				Node::Sequence(seq_at)
			}
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
			namespace.pop();

			// Create the result from the top namespace. Recognize various types:
			if let Some(node) = scene.mappings[name_at].fields.get("data") {
				// Check that data is actually a sequence holding objects
				match node {
					Node::Sequence(idx) => {
						let seq = &scene.sequences[*idx];
						for i in 0..seq.vals.len() {
							match seq.vals[i] {
								Node::Number(_) => {
									return Err(format!(
										"All elements in `data` must be objects, but a number was \
										 found at index {i}!"
									));
								},
								Node::Bool(_) => {
									return Err(format!(
										"All elements in `data` must be objects, but a boolean \
										 was found at index {i}!"
									));
								},
								Node::Sequence(_) => {
									return Err(format!(
										"All elements in `data` must be objects, but a sequence \
										 was found at index {i}!"
									));
								},
								_ => {},
							}
						}
					},
					_ => {
						return Err("Field `data` must be a sequence!".to_string());
					},
				}
				Node::Mapping(name_at)
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
						match value {
							Node::Number(_) => {
								return Err("Field `instance` must hold the value of some other \
								            object, not a number!"
									.to_string());
							},
							Node::Bool(_) => {
								return Err("Field `instance` must hold the value of some other \
								            object, not a bool!"
									.to_string());
							},
							Node::Sequence(_) => {
								return Err("Field `instance` must hold the value of some other \
								            object, not a sequence!"
									.to_string());
							},
							_ => {},
						}
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
				Node::Instance(scene_at)
			} else if scene.mappings[name_at].fields.contains_key("origin")
				&& scene.mappings[name_at].fields.contains_key("direction")
				&& scene.mappings[name_at].fields.contains_key("extent")
			{
				// This is actually a ray
				let mut origin = new_point(1.0);
				let mut direction = new_point(1.0);
				let mut extent = 0.0;
				let mut fields = HashMap::new();

				for (key, value) in scene.mappings[name_at].fields.iter() {
					if key == "origin" {
						origin = as_3d(scene, value)?;
					} else if key == "direction" {
						direction = as_3d(scene, value)?;
					} else if key == "extent" {
						match value {
							Node::Number(val) => {
								extent = *val;
							},
							_ => {
								return Err("Field `extent` in ray must be a float!".to_string());
							},
						}
					} else {
						fields.insert(key.clone(), *value);
					}
				}
				let ray = Ray {
					origin,
					direction,
					extent,
					fields,
				};
				let ray_at = scene.rays.len();
				scene.rays.push(ray);
				// We can safely remove the old mapping since we parsed it directly (and therefore,
				// it couldn't have saved and referenced elsewhere).
				scene.mappings.pop();
				Node::Ray(ray_at)
			} else {
				Node::Mapping(name_at)
			}
		},
		_ => return Err("Unsupported YAML value found while parsing scene data!".to_string()),
	};
	Ok(ret)
}

use nalgebra::matrix;
use yaml_rust2::Yaml;
pub fn to_ir(input: &Yaml) -> Result<Scene, String> {
	let mut scene = Scene {
		world: Node::Bool(false),
		sequences: vec![],
		strips: vec![],
		rays: vec![],
		instances: vec![],
		mappings: vec![],
	};

	let mut namespace: Vec<usize> = vec![];
	scene.world = parse(input, &mut namespace, &mut scene)?;

	Ok(scene)
}
