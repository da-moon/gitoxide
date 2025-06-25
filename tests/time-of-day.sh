#!/bin/bash
set -eu -o pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"
source "$SCRIPT_DIR/utilities.sh"
SUCCESSFULLY=0

COMMITTERS=(
  "Sebastian Thiel"
  "Eliah Kagan"
  "Edward Shen"
)

snapshot="$SCRIPT_DIR/snapshots/time-of-day"

(title "time-of-day" && \
  repo_root="$PWD" && \
  (
    sandbox && (
      git init &&
      git checkout -b main &&
      git config commit.gpgsign false &&
      git config tag.gpgsign false &&
      touch a && git add a && \
      GIT_AUTHOR_NAME="${COMMITTERS[0]}" GIT_AUTHOR_EMAIL=a@example.com \
      GIT_COMMITTER_NAME="${COMMITTERS[0]}" GIT_COMMITTER_EMAIL=a@example.com git commit -m first --date "2020-01-01T00:00:00 +0000" &&
      echo hi >> a && git add a && \
      GIT_AUTHOR_NAME="${COMMITTERS[1]}" GIT_AUTHOR_EMAIL=b@example.com \
      GIT_COMMITTER_NAME="${COMMITTERS[1]}" GIT_COMMITTER_EMAIL=b@example.com git commit -m second --date "2020-01-01T12:00:00 +0000" &&
      echo more >> a && git add a && \
      GIT_AUTHOR_NAME="${COMMITTERS[2]}" GIT_AUTHOR_EMAIL=c@example.com \
      GIT_COMMITTER_NAME="${COMMITTERS[2]}" GIT_COMMITTER_EMAIL=c@example.com git commit -m third --date "2020-01-01T23:00:00 +0000"
    )
    export REPO_ROOT="$repo_root"
    it "prints commit time histogram" && {
      WITH_SNAPSHOT="$snapshot/default" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- time-of-day --working-dir \"$PWD\" 2>/dev/null)"
    }
    idx=0
    for name in "${COMMITTERS[@]}"; do
      file="$(echo "$name" | tr ' ' '-')"
      it "filters by author $name" && {
        WITH_SNAPSHOT="$snapshot/$file" \
        expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- time-of-day --working-dir \"$PWD\" --author \"$name\" 2>/dev/null)"
      }
      idx=$((idx+1))
    done
  )
)
