use nalgebra::matrix;

use crate::ir::{Node, Point3D, Scene, TransformMat, as_3d, homogenize, homogenize_pt, new_point};

impl Node {
	pub fn set_bounds(
		&self,
		scene: &mut Scene,
		min: &mut Point3D,
		max: &mut Point3D,
		transform: &TransformMat,
	) {
		match self {
			Node::Strip(idx) => {
				let strip = &scene.strips[*idx];
				for vert in strip.vals.iter() {
					let point = transform * homogenize_pt(vert);
					for i in 0..3 {
						min[i] = f64::min(min[i], point[i]);
						max[i] = f64::max(max[i], point[i]);
					}
				}
			},
			Node::Ray(idx) => {
				let ray = &scene.rays[*idx];
				let rmin = new_point(ray.min);
				let extent = new_point(ray.extent);
				let start = ray.origin + ray.direction.component_mul(&rmin);
				let end = ray.origin + ray.direction.component_mul(&extent);

				let origin = transform * homogenize_pt(&start);
				let dest = transform * homogenize_pt(&end);

				for i in 0..3 {
					min[i] = f64::min(min[i], f64::min(origin[i], dest[i]));
					max[i] = f64::max(max[i], f64::max(origin[i], dest[i]));
				}
			},
			Node::Instance(idx) => {
				let instance = &scene.instances[*idx];
				let homogenous = &homogenize(transform);
				let mult = instance.obj_to_world() * homogenous;
				let affected = scene.instances[*idx].affected;
				affected.set_bounds(scene, min, max, &mult);
			},
			Node::Mapping(idx) => {
				// let mut map = &scene.instances[*idx];
				let mut mins = new_point(f64::NAN);
				let mut maxs = new_point(f64::NAN);

				let map = &scene.mappings[*idx];

				if let Some(n) = map.fields.get("min") {
					if let Ok(pt) = as_3d(scene, n) {
						for i in 0..3 {
							mins[i] = f64::min(mins[i], pt[i]);
							maxs[i] = f64::max(maxs[i], pt[i]);
						}
					}
				}

				if let Some(n) = map.fields.get("max") {
					if let Ok(pt) = as_3d(scene, n) {
						for i in 0..3 {
							mins[i] = f64::min(mins[i], pt[i]);
							maxs[i] = f64::max(maxs[i], pt[i]);
						}
					}
				}

				if let Some(Node::Sequence(idx)) = map.fields.get("data") {
					let seq = &scene.sequences[*idx];
					for element in seq.vals.clone() {
						element.set_bounds(scene, &mut mins, &mut maxs, transform);
					}
				}

				// If this mapping has dimensions, then it qualifies as a box
				// Checking x for NaN is the same as checking any for NaN. If any max or min is set,
				// then all must be set to some initial value. In other words, we cannot selectively
				// set some channels but not all.
				if !mins.x.is_nan() {
					let map = &mut scene.mappings[*idx];
					map.as_box(&mins, &maxs);

					// Update the parent box from this's dimensions
					for i in 0..3 {
						min[i] = f64::min(mins[i], min[i]);
						max[i] = f64::max(maxs[i], max[i]);
					}
				}
			},
			_ => {},
		}
	}
}

pub fn transform(
	scene: &mut Scene,
	root: bool,
	wrap: bool,
	box_size: u8,
	double: bool,
	triangle: bool,
) {
	if root {
		let old_world = scene.world;
		let should_box = match old_world {
			Node::Mapping(idx) => {
				let map = &mut scene.mappings[idx];
				// If the map wasn't activated previously, activate it now and be done
				map.is_box = true;
				false
			},
			// TODO: world root must be an object
			Node::Number(_) => panic!("Cannot box number root!"),
			Node::Bool(_) => panic!("Cannot box bool root!"),
			_ => true,
		};
		if should_box {
			todo!(
				"I think that mapping should have the data field directly, meaning that we don't \
				 have to make a sequence to set this."
			);
		}
	}

	if wrap {
		// Box any instance children which aren't boxes
		todo!();
	}

	if box_size != 0 {
		// Split any box which has too many children
		todo!();
	}

	if double {
		todo!();
	}

	if triangle {
		// todo!();
	}

	// The last transformation is to add box data to mappings where necessary
	let mut mins = new_point(f64::NAN);
	let mut maxs = new_point(f64::NAN);
	let transform = matrix![
		1.0, 0.0, 0.0, 0.0;
		0.0, 1.0, 0.0, 0.0;
		0.0, 0.0, 1.0, 0.0;
	];
	let world = scene.world;
	world.set_bounds(scene, &mut mins, &mut maxs, &transform);
}
