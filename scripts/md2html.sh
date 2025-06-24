#!/bin/bash

# Simple markdown to HTML converter
# Usage: ./md2html.sh file.md

if [ $# -eq 0 ]; then
    echo "Usage: $0 <file.md>"
    exit 1
fi

INPUT_FILE="$1"
OUTPUT_FILE="${INPUT_FILE%.md}.html"

if pandoc "$INPUT_FILE" -o "$OUTPUT_FILE" \
  --standalone \
  --highlight-style=breezedark \
  --metadata charset=utf-8 \
  -H <(echo '<style>body{background:black!important;color:white!important;font-family:-apple-system,sans-serif;max-width:100vw!important;line-height:1.6em;}pre{background:#333!important;color:white!important;padding:1em;overflow-x:auto}code{background:#333;color:magenta;padding:2px 4px}h1,h2,h3{color:magenta}</style>'); then
  echo "✓ $INPUT_FILE -> $OUTPUT_FILE"
else
  echo "✗ Failed to convert $INPUT_FILE"
  exit 1
fi
