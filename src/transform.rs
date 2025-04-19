use crate::ir::{Node, Point3D, Scene, as_3d, homogenize_pt, new_point};

impl Node {
	pub fn set_bounds(&self, scene: &mut Scene) -> (Point3D, Point3D) {
		match self {
			Node::Strip(idx) => {
				let strip = &scene.strips[*idx];
				let mut mins = new_point(f64::NAN);
				let mut maxs = new_point(f64::NAN);
				for vert in strip.vals.iter() {
					for i in 0..3 {
						mins[i] = f64::min(mins[i], vert[i]);
						maxs[i] = f64::max(maxs[i], vert[i]);
					}
				}
				(mins, maxs)
			},
			Node::Ray(idx) => {
				let ray = &scene.rays[*idx];
				let min = new_point(ray.min);
				let extent = new_point(ray.extent);
				let start = ray.origin + ray.direction.component_mul(&min);
				let end = ray.origin + ray.direction.component_mul(&extent);

				let mins = Point3D::new(
					f64::min(start.x, end.x),
					f64::min(start.y, end.y),
					f64::min(start.z, end.z),
				);
				let maxs = Point3D::new(
					f64::max(start.x, end.x),
					f64::max(start.y, end.y),
					f64::max(start.z, end.z),
				);
				(mins, maxs)
			},
			Node::Instance(idx) => {
				let affected = scene.instances[*idx].affected;
				let (mins, maxs) = affected.set_bounds(scene);
				let inst = &scene.instances[*idx];
				let transform = inst.obj_to_world();
				// Apply the transformation on the mins and maxs to get the true values
				let nmin = transform * homogenize_pt(&mins);
				let nmax = transform * homogenize_pt(&maxs);
				(nmin, nmax)
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

pub fn transform(scene: &mut Scene, root: bool, wrap: bool, box_size: u8, double: bool) {
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

	// The last transformation is to add box data to mappings where necessary
	let world = scene.world;
	world.set_bounds(scene);
}
