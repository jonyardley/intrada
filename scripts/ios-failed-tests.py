#!/usr/bin/env python3
"""Lists the failed test cases in an .xcresult, one `-only-testing:` flag per
line on stdout, and each one's failure messages on stderr.

A retry reruns only these, so one flake costs one test instead of the whole
UI suite running twice (#2269), and the message reaches the log, where a
green retry used to leave nothing to read (#1576)."""

import json
import os
import subprocess
import sys


def failed_cases(node):
    if node.get("nodeType") == "Test Case" and node.get("result") == "Failed":
        yield node
    for child in node.get("children", []):
        yield from failed_cases(child)


def messages(node):
    for child in node.get("children", []):
        if child.get("nodeType") == "Failure Message":
            line = child.get("sourceLocation", {}).get("lineNumber")
            yield f"{child['name']} (line {line})" if line else child["name"]
        else:
            yield from messages(child)


def only_testing(case):
    # The URL's last three parts are what `-only-testing:` takes: an XCTest
    # method bare, a Swift Testing function with its brackets.
    target, suite, test = case["nodeIdentifierURL"].split("/")[-3:]
    return f"-only-testing:{target}/{suite}/{test}"


def main(path):
    tree = json.loads(
        subprocess.run(
            ["xcrun", "xcresulttool", "get", "test-results", "tests", "--path", path],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    annotate = os.environ.get("GITHUB_ACTIONS") == "true"
    for root in tree["testNodes"]:
        for case in failed_cases(root):
            found = list(dict.fromkeys(messages(case)))
            print(f"✗ {case['nodeIdentifier']} ({case.get('duration', '?')})", file=sys.stderr)
            for message in found:
                print(f"    {message}", file=sys.stderr)
            if annotate:
                body = " / ".join(found).replace("%", "%25").replace("\n", "%0A")
                print(f"::warning title=Failed on the first run::{case['nodeIdentifier']}: {body}", file=sys.stderr)
            print(only_testing(case))


if __name__ == "__main__":
    main(sys.argv[1])
