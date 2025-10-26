#!/usr/bin/env bash
set -euo pipefail

export NAVI_HOME
NAVI_HOME="$(cd "$(dirname "$0")/.." && pwd)"

"${NAVI_HOME}/tests/run.sh"
