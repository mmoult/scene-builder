use crate::ir::{Node, Scene};

enum MapType {
	Unused,
	Box(usize),
	Procedural(usize),
}

fn to_major_minor(
	node: &Node,
	mappings: &[MapType],
	disabled_insts: &[usize],
) -> Option<(usize, usize)> {
	match node {
		Node::Strip(idx) => Some((2, *idx)),
		Node::Instance(idx) => {
			let mut delta = 0;
			for disabled in disabled_insts {
				match (*disabled).cmp(idx) {
					std::cmp::Ordering::Less => {
						delta += 1;
					},
					std::cmp::Ordering::Equal => return None,
					std::cmp::Ordering::Greater => break,
				}
			}
			Some((1, *idx - delta))
		},
		Node::Mapping(idx) => match mappings[*idx] {
			MapType::Unused => None,
			MapType::Box(i) => Some((0, i)),
			MapType::Procedural(i) => Some((3, i)),
		},
		_ => None,
	}
}

pub fn to_bvh(scene: &Scene) -> Vec<String> {
	// We need to check some conditions about mappings and instances before we can start printing

	// 1) Determine how to handle each mapping. Each can be one of: ignored, box, procedural. We
	//    must know the category each fits in before we start printing any nodes.
	let mut mappings = vec![];

	let mut box_num = 0;
	let mut boxes = vec![];
	let mut proc_num = 0;
	let mut procs = vec![];

	for i in 0..scene.mappings.len() {
		let mapping = &scene.mappings[i];
		if mapping.is_box {
			if mapping.fields.contains_key("min") {
				mappings.push(MapType::Procedural(proc_num));
				procs.push(i);
				proc_num += 1;
			} else {
				mappings.push(MapType::Box(box_num));
				boxes.push(i);
				box_num += 1;
			}
		} else {
			mappings.push(MapType::Unused);
		}
	}

	// 2) Rays are removed in the BVH target, so we must delete any instance nodes which have ray
	//    children (since they cannot exist independently).
	let mut disabled_insts = vec![];
	for (inst_idx, instance) in scene.instances.iter().enumerate() {
		if let Node::Ray(_) = instance.affected {
			disabled_insts.push(inst_idx);
		}
	}

	// Finally, print all nodes, using the numbering determined before to convert all references
	let mut res = vec!["{".to_string()];
	match to_major_minor(&scene.world, &mappings, &disabled_insts) {
		Some((major, minor)) => {
			res.push(format!("\t\"tlas\" : [ {}, {} ],", major, minor));
		},
		None => {
			res.push("}".to_string());
			return res;
		},
	};

	res.push("\t\"box_nodes\" : [".to_string());
	for (i, box_idx) in boxes.iter().enumerate() {
		res.push("\t\t{".to_string());
		let boxx = &scene.mappings[*box_idx];

		res.push(format!(
			"\t\t\t\"min_bounds\" : [ {}, {}, {} ],",
			boxx.min.x, boxx.min.y, boxx.min.z
		));
		res.push(format!(
			"\t\t\t\"max_bounds\" : [ {}, {}, {} ],",
			boxx.max.x, boxx.max.y, boxx.max.z
		));

		res.push("\t\t\t\"child_nodes\" : [".to_string());
		if let Some(Node::Sequence(idx)) = scene.mappings[*box_idx].fields.get("data") {
			let data = &scene.sequences[*idx];
			let mut kids = vec![];
			for node in data.vals.iter() {
				if let Some((major, minor)) = to_major_minor(node, &mappings, &disabled_insts) {
					kids.push((major, minor));
				}
			}
			let end = kids.len();
			for (i, (major, minor)) in kids.iter().enumerate() {
				if i + 1 == end {
					res.push(format!("\t\t\t\t[ {}, {} ]", major, minor));
				} else {
					res.push(format!("\t\t\t\t[ {}, {} ],", major, minor));
				}
			}
		}
		res.push("\t\t\t]".to_string());

		if i + 1 == boxes.len() {
			res.push("\t\t}".to_string());
		} else {
			res.push("\t\t},".to_string());
		}
	}
	res.push("\t],".to_string());

	res.push("\t\"instance_nodes\" : [".to_string());
	for (inst_idx, instance) in scene.instances.iter().enumerate() {
		// If this is an instance of a ray, do NOT print it!
		let cur = Node::Instance(inst_idx);
		if to_major_minor(&cur, &mappings, &disabled_insts).is_none() {
			continue;
		}

		res.push("\t\t{".to_string());

		let trans = instance.world_to_obj();
		res.push("\t\t\t\"world_to_object\" : [".to_string());
		for i in 0..4 {
			if i == 3 {
				res.push(format!(
					"\t\t\t\t[ {}, {}, {} ]",
					trans[(0, i)],
					trans[(1, i)],
					trans[(2, i)]
				))
			} else {
				res.push(format!(
					"\t\t\t\t[ {}, {}, {} ],",
					trans[(0, i)],
					trans[(1, i)],
					trans[(2, i)]
				))
			}
		}
		res.push("\t\t\t],".to_string());

		match to_major_minor(&instance.affected, &mappings, &disabled_insts) {
			Some((major, minor)) => {
				res.push(format!("\t\t\t\"child_node\" : [ {}, {} ],", major, minor));
			},
			None => panic!("Instance without legal child should have already been filtered!"),
		};

		let mut id = 0;
		if let Some(Node::Number(v)) = instance.fields.get("id") {
			id = *v as usize;
		}
		res.push(format!("\t\t\t\"id\" : {id},"));

		let mut custom_index = 0;
		if let Some(Node::Number(v)) = instance.fields.get("custom_index") {
			custom_index = *v as usize;
		}
		res.push(format!("\t\t\t\"custom_index\" : {custom_index},"));

		let mut mask = 255;
		if let Some(Node::Number(v)) = instance.fields.get("mask") {
			mask = *v as usize;
		}
		res.push(format!("\t\t\t\"mask\" : {mask},"));

		let mut sbt_record_offset = 0;
		if let Some(Node::Number(v)) = instance.fields.get("sbt_record_offset") {
			sbt_record_offset = *v as usize;
		}
		res.push(format!("\t\t\t\"sbt_record_offset\" : {sbt_record_offset}"));

		if inst_idx + 1 == scene.instances.len() {
			res.push("\t\t}".to_string());
		} else {
			res.push("\t\t},".to_string());
		}
	}
	res.push("\t],".to_string());

	res.push("\t\"triangle_nodes\" : [".to_string());
	for (tri_idx, tri) in scene.strips.iter().enumerate() {
		res.push("\t\t{".to_string());

		let mut geom_index = 0;
		if let Some(Node::Number(v)) = tri.fields.get("geometry_index") {
			geom_index = *v as usize
		}
		res.push(format!("\t\t\t\"geometry_index\" : {geom_index},"));

		let mut prim_index = tri_idx;
		if let Some(Node::Number(v)) = tri.fields.get("primitive_index") {
			prim_index = *v as usize;
		}
		res.push(format!("\t\t\t\"primitive_index\" : {prim_index},"));

		let mut opaque = true;
		if let Some(Node::Bool(v)) = tri.fields.get("opaque") {
			opaque = *v;
		}
		res.push(format!("\t\t\t\"opaque\" : {opaque},"));

		res.push("\t\t\t\"vertices\" : [".to_string());
		for (i, vert) in tri.vals.iter().enumerate() {
			if i + 1 == tri.vals.len() {
				res.push(format!("\t\t\t\t[ {}, {}, {} ]", vert.x, vert.y, vert.z));
			} else {
				res.push(format!("\t\t\t\t[ {}, {}, {} ],", vert.x, vert.y, vert.z));
			}
		}
		res.push("\t\t\t]".to_string());

		if tri_idx + 1 == scene.strips.len() {
			res.push("\t\t}".to_string());
		} else {
			res.push("\t\t},".to_string());
		}
	}
	res.push("\t],".to_string());

	res.push("\t\"procedural_nodes\" : [".to_string());
	for (i, proc_idx) in procs.iter().enumerate() {
		res.push("\t\t{".to_string());
		let proc = &scene.mappings[*proc_idx];

		res.push(format!(
			"\t\t\t\"min_bounds\" : [ {}, {}, {} ]",
			proc.min.x, proc.min.y, proc.min.z
		));
		res.push(format!(
			"\t\t\t\"max_bounds\" : [ {}, {}, {} ]",
			proc.max.x, proc.max.y, proc.max.z
		));

		let mut opaque = false;
		if let Some(Node::Bool(v)) = proc.fields.get("opaque") {
			opaque = *v;
		}
		res.push(format!("\t\t\t\"opaque\" : {opaque},"));

		let mut geom_index = 0;
		if let Some(Node::Number(v)) = proc.fields.get("geometry_index") {
			geom_index = *v as usize
		}
		res.push(format!("\t\t\t\"geometry_index\" : {geom_index},"));

		let mut prim_index = *proc_idx;
		if let Some(Node::Number(v)) = proc.fields.get("primitive_index") {
			prim_index = *v as usize;
		}
		res.push(format!("\t\t\t\"primitive_index\" : {prim_index}"));

		if i + 1 == procs.len() {
			res.push("\t\t}".to_string());
		} else {
			res.push("\t\t},".to_string());
		}
	}
	res.push("\t]".to_string());

	res.push("}".to_string());
	res
}
