#!/bin/bash
set -eu -o pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"
source "$SCRIPT_DIR/utilities.sh"
SUCCESSFULLY=0

snapshot="$SCRIPT_DIR/snapshots/churn"

(title "churn" && \
  repo_root="$PWD" && \
  (
    sandbox && (
      git init &&
      git checkout -b main &&
      git config commit.gpgsign false &&
      git config tag.gpgsign false &&
      echo a > a && git add a && \
      GIT_AUTHOR_NAME="Alice" GIT_AUTHOR_EMAIL=a@example.com \
      GIT_COMMITTER_NAME="Alice" GIT_COMMITTER_EMAIL=a@example.com git commit -m first &&
      echo b >> a && git add a && \
      GIT_AUTHOR_NAME="Bob" GIT_AUTHOR_EMAIL=b@example.com \
      GIT_COMMITTER_NAME="Bob" GIT_COMMITTER_EMAIL=b@example.com git commit -m second &&
      echo c > b && git add b && \
      GIT_AUTHOR_NAME="Alice" GIT_AUTHOR_EMAIL=a@example.com \
      GIT_COMMITTER_NAME="Alice" GIT_COMMITTER_EMAIL=a@example.com git commit -m third
    )
    export REPO_ROOT="$repo_root"
    it "prints churn by author" && {
      WITH_SNAPSHOT="$snapshot/author" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- churn --working-dir \"$PWD\" 2>/dev/null)"
    }
    it "prints churn per file" && {
      WITH_SNAPSHOT="$snapshot/file" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- churn --working-dir \"$PWD\" --per-file 2>/dev/null)"
    }
  )
)
