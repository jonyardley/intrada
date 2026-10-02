#!/usr/bin/env python3
"""Splits test classes across simulators by their last measured duration, and
records those durations from a result bundle for the next run.

    ios-ui-shards.py classes LISTING
        Each Target/Class in xcodebuild's `-enumerate-tests` JSON, once.
    ios-ui-shards.py count LISTING
        How many tests that listing enables.
    ios-ui-shards.py plan SHARDS SECONDS_FILE < classes
        One line per simulator: its `-only-testing:` flags, or empty.
    ios-ui-shards.py record BUNDLE SECONDS_FILE
        Adds each class's duration in BUNDLE to SECONDS_FILE.

xcodebuild's own clones took 60 to 86s to run their first test and one of six
never started, so the CI gate runs one xcodebuild per booted simulator instead
and does the dealing itself (#2269). A class is the unit, as it is for Xcode:
its tests share setup and order. Longest first onto the least loaded
simulator; a class with no measurement counts as the median of those with
one, so a new class neither jumps the queue nor hides at the end."""

import json
import statistics
import subprocess
import sys

# The middle of the UI classes measured on the CI Mac on 2026-10-02 (#2269).
UNMEASURED = 30.0


def read_seconds(path):
    seconds = {}
    try:
        with open(path) as f:
            for line in f:
                name, _, value = line.rstrip("\n").partition("\t")
                try:
                    seconds[name] = float(value)
                except ValueError:
                    continue
    except FileNotFoundError:
        pass
    return seconds


def plan(shards, seconds, classes):
    known = [seconds[c] for c in classes if c in seconds]
    default = statistics.median(known) if known else UNMEASURED
    load = [0.0] * shards
    dealt = [[] for _ in range(shards)]
    for c in sorted(set(classes), key=lambda c: (-seconds.get(c, default), c)):
        i = min(range(shards), key=lambda i: (load[i], i))
        load[i] += seconds.get(c, default)
        dealt[i].append(c)
    return dealt


def enabled(listing):
    found = []

    def walk(node):
        if isinstance(node, dict):
            found.extend(t.get("identifier", "") for t in node.get("enabledTests", []))
            for value in node.values():
                walk(value)
        elif isinstance(node, list):
            for value in node:
                walk(value)

    walk(listing)
    return found


def classes(listing):
    parts = (test.split("/") for test in enabled(listing))
    return list(dict.fromkeys("/".join(p[:2]) for p in parts if len(p) >= 2))


def durations(tests):
    """Seconds per Target/Class, from `xcresulttool get test-results tests`."""
    found = {}

    def walk(node):
        if node.get("nodeType") == "Test Case":
            parts = node.get("nodeIdentifierURL", "").split("/")
            if len(parts) >= 7 and node.get("durationInSeconds") is not None:
                key = f"{parts[4]}/{parts[5]}"
                found[key] = found.get(key, 0.0) + node["durationInSeconds"]
            return
        for child in node.get("children", []):
            walk(child)

    for root in tests.get("testNodes", []):
        walk(root)
    return found


def record(bundle, path):
    tests = json.loads(
        subprocess.run(
            ["xcrun", "xcresulttool", "get", "test-results", "tests", "--path", bundle],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    seconds = read_seconds(path)
    seconds.update(durations(tests))
    with open(path, "w") as f:
        for name in sorted(seconds):
            f.write(f"{name}\t{seconds[name]:.1f}\n")


def main(argv):
    if len(argv) == 3 and argv[1] in ("classes", "count"):
        with open(argv[2]) as f:
            listing = json.load(f)
        if argv[1] == "count":
            print(len(enabled(listing)))
        else:
            for c in classes(listing):
                print(c)
        return 0
    if len(argv) == 4 and argv[1] == "plan":
        names = [line.strip() for line in sys.stdin if line.strip()]
        for shard in plan(int(argv[2]), read_seconds(argv[3]), names):
            print(" ".join(f"-only-testing:{c}" for c in shard))
        return 0
    if len(argv) == 4 and argv[1] == "record":
        record(argv[2], argv[3])
        return 0
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
