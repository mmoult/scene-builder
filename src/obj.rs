use std::collections::HashSet;

use crate::ir::{Node, Point3D, Scene};
use crate::report::warn;
use nalgebra::matrix;

fn update_color(
	new: Option<&Node>,
	lines: &mut Vec<String>,
	scene: &Scene,
	color: &mut usize,
	colors: &mut HashSet<usize>,
) {
	match new {
		None => {},
		Some(node) => {
			match node {
				Node::Sequence(idx) => {
					if *idx == *color && !colors.is_empty() {
						return; // color already matches!
					}
					*color = *idx;
					if colors.contains(idx) {
						// Already registered color, switch to it and be done
						lines.push(format!("usemtl color{}", *idx));
						return;
					}

					// Otherwise, register the new color
					let vals = &scene.sequences[*idx].vals;
					let len = vals.len();
					if len != 3 {
						warn(&format!(
							"`color` is expected to have 3 components! {len} found instead."
						))
					}
					let mut fcolor = Point3D::new(0.0, 0.0, 0.0);
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
					lines.push("".to_string());
					lines.push(format!("newmtl color{}", *idx));
					lines.push(format!("Kd {} {} {}", fcolor.x, fcolor.y, fcolor.z));
					lines.push("Ks 0.5 0.5 0.5".to_string());
					lines.push("Ns 18.0".to_string());
					lines.push("".to_string());
					lines.push(format!("usemtl color{}", *idx));
					// Save so we can use it again
					colors.insert(*idx);
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
}

use crate::ir::TransformMat;

fn handle_node(
	node: &Node,
	lines: &mut Vec<String>,
	scene: &Scene,
	color: &mut usize,
	colors: &mut HashSet<usize>,
	transform: &TransformMat,
) {
	match node {
		Node::Strip(idx) => todo!(),
		Node::Ray(idx) => {
			let ray = &scene.rays[*idx];
			update_color(ray.fields.get("color"), lines, scene, color, colors);
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
			update_color(instance.fields.get("color"), lines, scene, color, colors);
			// Instance doesn't push any lines, but it does update the transformation matrix
			todo!();
		},
		Node::Mapping(idx) => {
			let map = &scene.mappings[*idx];
			let color_now = map.fields.get("color");
			if let Some(Node::Sequence(idx)) = map.fields.get("data") {
				let seq = &scene.sequences[*idx];
				for node in seq.vals.iter() {
					update_color(color_now, lines, scene, color, colors);
					handle_node(node, lines, scene, color, colors, transform);
				}
			}
		},
		_ => {}, // For non-objects encountered alone, we are missing the required context to print
	}
}

pub fn to_obj(scene: &Scene) -> Vec<String> {
	// Append header to every obj file
	let mut res = vec!["# Recommended viewer: https://3dviewer.net/".to_string()];
	let mut color = 0;
	let mut colors = HashSet::new();
	let transform = matrix![
		1.0, 0.0, 0.0;
		0.0, 1.0, 0.0;
		0.0, 0.0, 1.0;
		0.0, 0.0, 0.0;
	];
	handle_node(
		&scene.world,
		&mut res,
		scene,
		&mut color,
		&mut colors,
		&transform,
	);
	res
}
