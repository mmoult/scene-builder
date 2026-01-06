#!/usr/bin/env python3
import argparse
import os

#=======================================================================================================================
# Verify that all outputs in `examples` match what the scene builder currently generates.
#=======================================================================================================================

parser = argparse.ArgumentParser(description="Run all integration tests for scene-builder.")
parser.add_argument("--regen", "-r", action="store_true", help="Regenerate expected output files.")
parser.add_argument("--verbose", "-v", action="store_true", help="Print all found tests and launch command.")
args = parser.parse_args()

# Check that the interpreter has been built
repo_root = os.path.abspath(os.path.dirname(__file__))
build_path = os.path.join(repo_root, "target")
# Look for build in `release` first, then `debug` if not present
BIN_NAME = "scene-builder"
release_bin = os.path.join(build_path, "release", BIN_NAME)
debug_bin = os.path.join(build_path, "debug", BIN_NAME)

if os.path.isfile(release_bin):
    use_bin = release_bin
elif os.path.isfile(debug_bin):
    use_bin = debug_bin
else:
    print("Could not find", BIN_NAME, "binary. Is it built?")
    print("Looking for `release` or `debug` directory within:", build_path)
    exit(1)

print("Found", BIN_NAME, "at:", use_bin)

def eq_file(got, expected_file):
    with open(expected_file, "rb") as f:
        seen = f.read()
    return seen == got

fails = 0
total = 0

example_path = os.path.join(repo_root, "examples")
import subprocess

def run(root, scene, out, format, regen, cmd_args):
    global fails, total
    scene = os.path.join(root, scene)
    out = os.path.join(root, out)
    total += 1
    cmd = [use_bin, "-f", format] + cmd_args + [scene]
    res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    status = None
    reports = []
    if res.returncode != 0 or not eq_file(res.stdout, out):
        fails += 1
        status = "X"
        if res.returncode != 0:
            reports.append("Non-zero exit code: {}".format(res.returncode))
        else:
            reports.append("Output differs from expected.")
    elif args.verbose:
        status = "✓"

    if status is not None:
        if args.verbose:
            print(status, " ".join(cmd))
            for report in reports:
                print(" ", report)
        else:
            print(status, os.path.relpath(out, example_path))

    if regen:
        with open(out, "w") as f:
            f.write(res.stdout.decode())

for (root, dirs, files) in os.walk(example_path, topdown=True):
    scene = None
    obj_out = None
    bvh_json_out = None
    cmd_args = []
    for file in files:
        if file.startswith("out."):
            if file.endswith(".obj"):
                obj_out = file
            elif file.endswith(".json"):
                bvh_json_out = file
        elif file.endswith(".yaml"):
            scene = file
        elif file == "args.txt":
            with open(os.path.join(root, file), "r") as f:
                cmd_args = f.read().strip().split()

    if scene is not None:
        if obj_out is not None:
            run(root, scene, obj_out, "obj", args.regen, cmd_args)
        if bvh_json_out is not None:
            run(root, scene, bvh_json_out, "bvh", args.regen, cmd_args)

# Print results
if total == 0:
    print("No tests run!")
    exit(1)
else:
    if fails == 0:
        print("PASS", end="")
    else:
        print("FAIL", end="")
print(": ", (total - fails), "/", total)
exit(fails)
