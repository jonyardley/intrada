#!/usr/bin/env python3
"""Lists the failed test cases in an .xcresult, one `-only-testing:` flag per
line on stdout, and each one's failure messages on stderr.

A retry reruns only these, so one flake costs one test instead of the whole
UI suite running twice (#2269), and the message reaches the log, where a
green retry used to leave nothing to read (#1576).

Prints no flags and exits 1 when the failures are not all accounted for by
failed test cases (a refused clone fails a class with no case under it): a
rerun of the cases alone would then forgive the rest."""

import json
import os
import subprocess
import sys


def xcresult(kind, path):
    return json.loads(
        subprocess.run(
            ["xcrun", "xcresulttool", "get", "test-results", kind, "--path", path],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )


def walk(node, cases, stray):
    """Collects failed cases; returns whether a failed case sits at or under node."""
    if node.get("nodeType") == "Test Case":
        if node.get("result") == "Failed":
            cases.append(node)
            return True
        return False
    if node.get("nodeType") == "Failure Message":
        stray.append(node.get("name", "?"))
        return False
    below = [walk(child, cases, stray) for child in node.get("children", [])]
    if node.get("result") == "Failed" and not any(below):
        stray.append(f"{node.get('nodeType')} {node.get('name')} failed with no failed test under it")
    return any(below)


def messages(node):
    for child in node.get("children", []):
        if child.get("nodeType") == "Failure Message":
            line = child.get("sourceLocation", {}).get("lineNumber")
            yield f"{child['name']} (line {line})" if line else child["name"]
        else:
            yield from messages(child)


def only_testing(case):
    # test://com.apple.xcode/<project>/<target>/<suite>[/<suite>...]/<test>
    # An XCTest method comes bare, a Swift Testing function with its brackets,
    # and both are what `-only-testing:` takes.
    parts = case.get("nodeIdentifierURL", "").split("/")
    if len(parts) < 7 or parts[:3] != ["test:", "", "com.apple.xcode"]:
        return None
    return "-only-testing:" + "/".join(parts[4:])


def main(path):
    cases, stray = [], []
    for root in xcresult("tests", path)["testNodes"]:
        walk(root, cases, stray)
    reported = xcresult("summary", path)["failedTests"]
    flags = [only_testing(case) for case in cases]

    annotate = os.environ.get("GITHUB_ACTIONS") == "true"
    for case in cases:
        found = list(dict.fromkeys(messages(case)))
        print(f"✗ {case['nodeIdentifier']} ({case.get('duration', '?')})", file=sys.stderr)
        for message in found:
            print(f"    {message}", file=sys.stderr)
        if annotate:
            body = " / ".join(found).replace("%", "%25").replace("\n", "%0A")
            print(f"::warning title=Failed on the first run::{case['nodeIdentifier']}: {body}", file=sys.stderr)
    for line in stray:
        print(f"✗ {line}", file=sys.stderr)

    if stray or reported != len(cases) or None in flags:
        print(
            f"✗ {reported} reported failed, {len(cases)} failed tests found, {len(stray)} failures "
            "outside a test: not every failure can be rerun, so none is.",
            file=sys.stderr,
        )
        return 1
    for flag in flags:
        print(flag)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
