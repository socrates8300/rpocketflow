#!/usr/bin/env bash
# ci-walden tenant-interface v4 — QA artifact drill (positive, rerun)
# contract: bash at repo root; exit status is the verdict (0 = green)
set -e
mkdir -p dist
printf 'artifact-drill qa-artifacts-lane run2 fixed-string v2\n' > dist/artifact.txt
echo "tenant gate ok"
