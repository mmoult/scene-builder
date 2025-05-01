use crate::ir::{Mapping, Node, Point3D, Scene, Sequence, as_3d, homogenize_pt, new_point};

impl Node {
	pub fn set_bounds(&self, scene: &mut Scene) -> (Point3D, Point3D) {
		match self {
			Node::Strip(idx) => {
				let strip = &scene.strips[*idx];
				let mut min = strip.vals[0];
				let mut max = strip.vals[0];
				for vert in strip.vals.iter().skip(1) {
					for i in 0..3 {
						min[i] = f64::min(min[i], vert[i]);
						max[i] = f64::max(max[i], vert[i]);
					}
				}
				(min, max)
			},
			Node::Ray(idx) => {
				let ray = &scene.rays[*idx];
				let rmin = new_point(ray.min);
				let extent = new_point(ray.extent);
				let start = ray.origin + ray.direction.component_mul(&rmin);
				let end = ray.origin + ray.direction.component_mul(&extent);

				let mut min = new_point(f64::NAN);
				let mut max = new_point(f64::NAN);

				for i in 0..3 {
					min[i] = f64::min(min[i], f64::min(start[i], end[i]));
					max[i] = f64::max(max[i], f64::max(start[i], end[i]));
				}
				(min, max)
			},
			Node::Instance(idx) => {
				let instance = &scene.instances[*idx];
				let mult = instance.obj_to_world();
				let affected = scene.instances[*idx].affected;
				let (amin, amax) = affected.set_bounds(scene);

				let mut min = new_point(f64::NAN);
				let mut max = new_point(f64::NAN);

				// Construct an axis-aligned bounding box around the min and max of the affected
				for i in 0..8 {
					let mut point = new_point(0.0);
					for j in 0..3 {
						point[j] = if ((i >> j) & 1) == 1 {
							amax[j]
						} else {
							amin[j]
						}
					}

					let vert = mult * homogenize_pt(&point);
					for j in 0..3 {
						min[j] = f64::min(min[j], vert[j]);
						max[j] = f64::max(max[j], vert[j]);
					}
				}

				(min, max)
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
						let (emin, emax) = element.set_bounds(scene);
						for i in 0..3 {
							mins[i] = f64::min(mins[i], emin[i]);
							maxs[i] = f64::max(maxs[i], emax[i]);
						}
					}
				}

				// If this mapping has dimensions, then it qualifies as a box
				// Checking x for NaN is the same as checking any for NaN. If any max or min is set,
				// then all must be set to some initial value. In other words, we cannot selectively
				// set some channels but not all.
				if !mins.x.is_nan() {
					let map = &mut scene.mappings[*idx];
					map.as_box(&mins, &maxs);
				}

				(mins, maxs)
			},
			_ => (new_point(f64::NAN), new_point(f64::NAN)),
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
		let should_box = match scene.world {
			Node::Mapping(_) => {
				// If the root is already a mapping, we cannot do anything more. If it has legal
				// children, then it will be made a box. If no legal children, then it wouldn't
				// make sense to box it further.
				false
			},
			// World root must be an object
			Node::Number(_) => panic!("Cannot box number root!"),
			Node::Bool(_) => panic!("Cannot box bool root!"),
			_ => true,
		};
		if should_box {
			let seq_at = scene.sequences.len();
			scene.sequences.push(Sequence::new());
			scene.sequences[seq_at].vals.push(scene.world);

			let name_at = scene.mappings.len();
			scene.mappings.push(Mapping::new());
			scene.mappings[name_at]
				.fields
				.insert("data".to_string(), Node::Sequence(seq_at));

			// Replace the old world reference with the newly created one
			scene.world = Node::Mapping(name_at);
		}
	}

	// Split tri-nodes with more than 3 vertices into individual triangles
	if triangle {
		// todo!();
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

	// The last transformation is to add box data to mappings where necessary
	let world = scene.world;
	world.set_bounds(scene);
}
