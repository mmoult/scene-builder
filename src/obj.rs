use std::collections::HashSet;

use crate::ir::{Node, Point3D, Scene, new_point};
use crate::report::warn;
use nalgebra::matrix;

struct Palette {
	pub current: usize,
	materials: HashSet<usize>,
}
impl Palette {
	pub fn new(lines: &mut Vec<String>, default: usize) -> Palette {
		let mut palette = Palette {
			current: 0,
			materials: HashSet::new(),
		};
		// Default color is black
		palette.register(lines, &new_point(0.0), default);
		palette
	}

	/// Emit the change to a previously defined color
	pub fn reuse(&mut self, lines: &mut Vec<String>, color: usize) {
		lines.push(format!("usemtl color{}", color));
		self.current = color;
	}

	/// Reuse the given color if it isn't current. Useful for resetting color after handling a child
	/// node
	pub fn reset(&mut self, lines: &mut Vec<String>, color: usize) {
		if self.current != color {
			self.reuse(lines, color);
		}
	}

	/// Register a new color. Does not check if the color has already been defined. For that, use `update`.
	fn register(&mut self, lines: &mut Vec<String>, color: &Point3D, idx: usize) {
		lines.push("".to_string());
		lines.push(format!("newmtl color{}", idx));
		lines.push(format!("Kd {} {} {}", color.x, color.y, color.z));
		lines.push("Ks 0.5 0.5 0.5".to_string());
		lines.push("Ns 18.0".to_string());
		lines.push("".to_string());
		lines.push(format!("usemtl color{}", idx));
		// Save so we can use it again
		self.materials.insert(idx);
		self.current = idx;
	}

	pub fn update(&mut self, new: Option<&Node>, lines: &mut Vec<String>, scene: &Scene) -> usize {
		match new {
			None => {},
			Some(node) => {
				match node {
					Node::Sequence(idx) => {
						// Verify that color isn't already current
						if *idx != self.current {
							// If the color is already registered, use that and be done
							if self.materials.contains(idx) {
								self.reuse(lines, *idx);
							} else {
								// Otherwise, register the new color
								let vals = &scene.sequences[*idx].vals;
								let len = vals.len();
								if len != 3 {
									warn(&format!(
										"`color` is expected to have 3 components! {len} found \
										 instead."
									))
								}
								let mut fcolor = new_point(0.0);
								for i in 0..std::cmp::min(3, len) {
									if let Node::Number(f) = vals[i] {
										fcolor[i] = f / 255.0;
									} else {
										warn(&format!(
											"`color` channel {} is expected to be a number!",
											i
										))
									}
								}
								self.register(lines, &fcolor, *idx);
							}
						}
					},
					_ => {
						warn(&format!(
							"`color` is not a sequence as expected! Got {} instead.",
							node
						));
					},
				}
			},
		}
		self.current
	}
}

use crate::ir::TransformMat;

fn handle_node(
	node: &Node,
	lines: &mut Vec<String>,
	scene: &Scene,
	palette: &mut Palette,
	transform: &TransformMat,
) {
	match node {
		Node::Strip(idx) => {
			let strip = &scene.strips[*idx];
			palette.update(strip.fields.get("color"), lines, scene);
			lines.push("".to_string());
			lines.push(format!("o strip{}", *idx));
			let mut inverse = false;
			let mut count = 0;
			for vert in strip.vals.iter() {
				lines.push(format!("v {} {} {}", vert.x, vert.y, vert.z));
				if count >= 2 {
					if inverse {
						lines.push("f -2 -3 -1".to_string());
					} else {
						lines.push("f -3 -2 -1".to_string());
					}
					inverse = !inverse;
				} else {
					count += 1;
				}
			}
		},
		Node::Ray(idx) => {
			let ray = &scene.rays[*idx];
			palette.update(ray.fields.get("color"), lines, scene);
			let origin = transform * ray.origin;
			let direction = transform * (ray.direction * ray.extent);
			let end = origin + direction;
			lines.push("".to_string());
			lines.push(format!("o ray{}", *idx));
			lines.push(format!("v {} {} {}", origin.x, origin.y, origin.z));
			lines.push(format!("v {} {} {}", end.x, end.y, end.z));
			lines.push("l -2 -1".to_string()); // line from penultimate vertex to ultimate
		},
		Node::Instance(idx) => {
			let instance = &scene.instances[*idx];
			palette.update(instance.fields.get("color"), lines, scene);
			// Instance doesn't push any lines, but it does update the transformation matrix
			let homogenous = &instance.homogenize(transform);
			let mult = homogenous * instance.obj_to_world();
			handle_node(&instance.affected, lines, scene, palette, &mult);
		},
		Node::Mapping(idx) => {
			let map = &scene.mappings[*idx];
			let color = palette.update(map.fields.get("color"), lines, scene);
			if map.is_box {
				// create a box if min and max are present
				lines.push("".to_string());
				lines.push(format!("o box{}", *idx));
				lines.push(format!("v {} {} {}", map.min.x, map.min.y, map.min.z));
				lines.push(format!("v {} {} {}", map.min.x, map.min.y, map.max.z));
				lines.push(format!("v {} {} {}", map.min.x, map.max.y, map.min.z));
				lines.push(format!("v {} {} {}", map.min.x, map.max.y, map.max.z));
				lines.push(format!("v {} {} {}", map.max.x, map.min.y, map.min.z));
				lines.push(format!("v {} {} {}", map.max.x, map.min.y, map.max.z));
				lines.push(format!("v {} {} {}", map.max.x, map.max.y, map.min.z));
				lines.push(format!("v {} {} {}", map.max.x, map.max.y, map.max.z));

				let mut fill = false;
				if let Some(Node::Bool(val)) = map.fields.get("fill") {
					fill = *val;
				}

				if fill {
					lines.push("f -8 -7 -5 -6".to_string()); // minX
					lines.push("f -8 -7 -3 -4".to_string()); // minY
					lines.push("f -7 -5 -1 -3".to_string()); // minZ
					lines.push("f -4 -3 -1 -2".to_string()); // maxX
					lines.push("f -6 -5 -1 -2".to_string()); // maxY
					lines.push("f -8 -6 -2 -4".to_string()); // maxZ
				} else {
					lines.push("l -8 -7 -5 -6".to_string());
					lines.push("l -3 -1 -2 -4".to_string());
					lines.push("l -8 -4 -3 -7 -5 -1 -2 -6 -8".to_string());
				}
			}
			if let Some(Node::Sequence(idx)) = map.fields.get("data") {
				let seq = &scene.sequences[*idx];
				for node in seq.vals.iter() {
					palette.reset(lines, color);
					handle_node(node, lines, scene, palette, transform);
				}
			}
		},
		_ => {}, // For non-objects encountered alone, we are missing the required context to print
	}
}

pub fn to_obj(scene: &Scene) -> Vec<String> {
	// Append header to every obj file
	let mut res = vec![
		"# Generated by Scene Builder @ https://github.com/mmoult/scene-builder".to_string(),
		"# Recommended OBJ viewer: https://3dviewer.net/".to_string(),
	];
	let transform = matrix![
		1.0, 0.0, 0.0;
		0.0, 1.0, 0.0;
		0.0, 0.0, 1.0;
		0.0, 0.0, 0.0;
	];
	let mut palette = Palette::new(&mut res, scene.sequences.len());
	handle_node(&scene.world, &mut res, scene, &mut palette, &transform);
	res
}
