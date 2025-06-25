#!/bin/bash
set -eu -o pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"
source "$SCRIPT_DIR/utilities.sh"
SUCCESSFULLY=0

snapshot="$SCRIPT_DIR/snapshots/commit-size"

(title "commit-size" && \
  repo_root="$PWD" && \
  (
    sandbox && (
      git init &&
      git checkout -b main &&
      git config commit.gpgsign false &&
      git config tag.gpgsign false &&
      echo a > f1 && git add f1 && git commit -m c1 &&
      echo b > f2 && echo b >> f2 && git add f2 && git commit -m c2 &&
      echo c > f3 && echo c >> f3 && echo c >> f3 && git add f3 && git commit -m c3
    )
    export REPO_ROOT="$repo_root"
    it "prints commit size stats" && {
      WITH_SNAPSHOT="$snapshot/default" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- commit-size --working-dir \"$PWD\" 2>/dev/null)"
    }
    it "prints percentiles" && {
      WITH_SNAPSHOT="$snapshot/percentiles" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- commit-size --working-dir \"$PWD\" --percentiles 50,100 2>/dev/null)"
    }
  )
)
