#!/bin/bash
set -eu -o pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"
source "$SCRIPT_DIR/utilities.sh"
SUCCESSFULLY=0

snapshot="$SCRIPT_DIR/snapshots/streaks"

(title "streaks" && \
  repo_root="$PWD" && \
  (
    sandbox && (
      git init &&
      git checkout -b main &&
      git config commit.gpgsign false &&
      git config tag.gpgsign false &&
      echo a > a && git add a && \
      GIT_AUTHOR_NAME="Alice" GIT_AUTHOR_EMAIL=a@example.com \
      GIT_COMMITTER_NAME="Alice" GIT_COMMITTER_EMAIL=a@example.com git commit -m a1 --date "2020-01-01T00:00:00 +0000" &&
      echo a >> a && git add a && \
      GIT_AUTHOR_NAME="Alice" GIT_AUTHOR_EMAIL=a@example.com \
      GIT_COMMITTER_NAME="Alice" GIT_COMMITTER_EMAIL=a@example.com git commit -m a2 --date "2020-01-02T00:00:00 +0000" &&
      echo a >> a && git add a && \
      GIT_AUTHOR_NAME="Alice" GIT_AUTHOR_EMAIL=a@example.com \
      GIT_COMMITTER_NAME="Alice" GIT_COMMITTER_EMAIL=a@example.com git commit -m a3 --date "2020-01-03T00:00:00 +0000" &&
      echo b > b && git add b && \
      GIT_AUTHOR_NAME="Bob" GIT_AUTHOR_EMAIL=b@example.com \
      GIT_COMMITTER_NAME="Bob" GIT_COMMITTER_EMAIL=b@example.com git commit -m b1 --date "2020-01-01T00:00:00 +0000" &&
      echo b >> b && git add b && \
      GIT_AUTHOR_NAME="Bob" GIT_AUTHOR_EMAIL=b@example.com \
      GIT_COMMITTER_NAME="Bob" GIT_COMMITTER_EMAIL=b@example.com git commit -m b2 --date "2020-01-04T00:00:00 +0000" &&
      echo a >> a && git add a && \
      GIT_AUTHOR_NAME="Alice" GIT_AUTHOR_EMAIL=a@example.com \
      GIT_COMMITTER_NAME="Alice" GIT_COMMITTER_EMAIL=a@example.com git commit -m a4 --date "2020-01-05T00:00:00 +0000"
    )
    export REPO_ROOT="$repo_root"
    it "prints streaks" && {
      WITH_SNAPSHOT="$snapshot/default" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- streaks --working-dir \"$PWD\" 2>/dev/null)"
    }
    it "filters author" && {
      WITH_SNAPSHOT="$snapshot/filtered" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- streaks --working-dir \"$PWD\" --author Alice 2>/dev/null)"
    }
  )
)
