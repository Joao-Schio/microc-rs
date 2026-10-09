"""Print source lines not executed by coverage tests (LCOV format)."""
import sys

file = None
missed = {}
for raw_line in open(sys.argv[1], encoding="utf-8"):
    line = raw_line.strip()
    if line.startswith("SF:"):
        file = line[3:]
    elif line.startswith("DA:") and file is not None and "/src/" in file:
        number, hits, *_ = line[3:].split(",")
        if int(hits) == 0:
            missed.setdefault(file.split("/src/")[-1], []).append(int(number))

for path, lines in sorted(missed.items(), key=lambda item: -len(item[1])):
    print(f"{path}: {len(lines)} uncovered lines: {', '.join(map(str, sorted(set(lines))))}")
