#!/usr/bin/env python3

import json
import sys
from pathlib import Path


def badge_color(percent: float) -> str:
    if percent >= 90:
        return "#4c1"
    if percent >= 80:
        return "#97ca00"
    if percent >= 70:
        return "#a4a61d"
    if percent >= 60:
        return "#dfb317"
    return "#e05d44"


def main() -> int:
    if len(sys.argv) != 3:
        print(
            "usage: generate_coverage_badge.py <coverage.json> <output.svg>",
            file=sys.stderr,
        )
        return 2

    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])

    report = json.loads(input_path.read_text())

    try:
        percent = float(report["data"][0]["totals"]["lines"]["percent"])
    except (KeyError, IndexError, TypeError, ValueError) as error:
        raise RuntimeError("coverage report does not contain line coverage totals") from error

    value = f"{percent:.1f}%"
    color = badge_color(percent)

    svg = f'''<svg xmlns="http://www.w3.org/2000/svg" width="150" height="20" role="img" aria-label="coverage: {value}">
  <title>coverage: {value}</title>
  <linearGradient id="s" x2="0" y2="100%">
    <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>
    <stop offset="1" stop-opacity=".1"/>
  </linearGradient>
  <clipPath id="r">
    <rect width="150" height="20" rx="3" fill="#fff"/>
  </clipPath>
  <g clip-path="url(#r)">
    <rect width="82" height="20" fill="#555"/>
    <rect x="82" width="68" height="20" fill="{color}"/>
    <rect width="150" height="20" fill="url(#s)"/>
  </g>
  <g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" font-size="11">
    <text x="41" y="15" fill="#010101" fill-opacity=".3">coverage</text>
    <text x="41" y="14">coverage</text>
    <text x="116" y="15" fill="#010101" fill-opacity=".3">{value}</text>
    <text x="116" y="14">{value}</text>
  </g>
</svg>
'''

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(svg)
    print(f"Line coverage: {value}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
