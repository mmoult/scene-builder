# Scene Builder

Describe a scene in a high-level language (see [scene-lang](examples/scene-lang.md)) which can be compiled into BVH
format (for use with [SPIRV-Interpreter](https://github.com/mmoult/SPIRV-Interpreter)) or
[OBJ format](https://en.wikipedia.org/wiki/Wavefront_.obj_file) for easy visualization.

Check out the various [examples](examples) to learn more.

## Contributing
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be licensed under the Apache License, Version 2.0, without any additional
terms or conditions.

All code contributions should pass all:
* quality checks (run `cargo clippy --fix --allow-dirty -- -D warnings`)
* formatting standards (run `cargo fmt --all`)
* unit tests (run `cargo test`)
* integration tests (build with `cargo build` then run `test.py`)

## License
The source code, test examples, and all other associated files are distributed under the Apache 2.0 license.

   Copyright 2025 [scene-builder](https://github.com/mmoult/scene-builder)

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
