#!/usr/bin/env bash
# ci-walden tenant-interface v4 — judge probe JP1 (positive: judge-designed tricky-name tree)
# contract: bash at repo root; exit status is the verdict (0 = green)
set -e
rm -rf dist
mkdir -p dist/deep/one/two/three
printf 'judge p1: deeply nested leaf\n' > dist/deep/one/two/three/leaf.txt
printf 'judge p1: spaces and parens\n' > 'dist/name with spaces and (parens).txt'
printf 'judge p1: embedded double quote\n' > 'dist/quote"in-name.txt'
printf 'judge p1: embedded backslash\n' > 'dist/back\slash.txt'
printf 'judge p1: embedded tab\n' > $'dist/tab\tin-name.txt'
printf 'judge p1: unicode name\n' > 'dist/ünïcode-名前.txt'
: > dist/zero-length.bin
printf 'judge p1: plain anchor\n' > dist/plain.txt
echo "tenant gate ok"
