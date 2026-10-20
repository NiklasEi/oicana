#!/usr/bin/env python3
"""Write the changelog of the Dart package from the changelog of the repository.

Keeps the general entries and the "Dart and Flutter" section of every version since
the package exists, and names the unreleased version after the one being published.

usage: dart-changelog.py <CHANGELOG.md> <version> <output>
"""

import re
import sys

SECTION = "Dart and Flutter"
FIRST_VERSION = (0, 10, 0)


def release(heading):
    match = re.match(r"v?(\d+)\.(\d+)\.(\d+)", heading)
    return tuple(int(part) for part in match.groups()) if match else None


def main(source, version, output):
    versions = []
    section = None
    for line in open(source, encoding="utf-8"):
        line = line.rstrip("\n")
        if line.startswith("## "):
            heading = line[3:].strip()
            number = release(heading)
            if number is not None and number < FIRST_VERSION:
                break
            if heading == "Unreleased":
                heading = f"v{version}"
            versions.append((heading, []))
            section = None
        elif line.startswith("### "):
            section = line[4:].strip()
        elif versions and section in (None, SECTION):
            versions[-1][1].append(line)

    with open(output, "w", encoding="utf-8") as out:
        out.write("# Changelog\n")
        for heading, lines in versions:
            entries = [line for line in lines if line.strip()]
            if entries:
                out.write(f"\n## {heading}\n\n" + "\n".join(entries) + "\n")


if __name__ == "__main__":
    if len(sys.argv) != 4:
        sys.exit(__doc__.strip().splitlines()[-1])
    main(*sys.argv[1:])
