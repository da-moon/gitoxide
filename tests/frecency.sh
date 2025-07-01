#!/bin/bash
set -eu -o pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"
source "$SCRIPT_DIR/utilities.sh"
SUCCESSFULLY=0

snapshot="$SCRIPT_DIR/snapshots/frecency"

(title "frecency" && \
  repo_root="$PWD" && \
  (
    sandbox && (
      git init &&
      git checkout -b main &&
      git config commit.gpgsign false &&
      git config tag.gpgsign false &&
      export GIT_AUTHOR_DATE="2020-01-01T00:00:00 +0000" &&
      export GIT_COMMITTER_DATE="$GIT_AUTHOR_DATE" &&
      echo a > file1 && git add file1 && git commit -m first --date "$GIT_AUTHOR_DATE" &&
      export GIT_AUTHOR_DATE="2020-01-02T00:00:00 +0000" &&
      export GIT_COMMITTER_DATE="$GIT_AUTHOR_DATE" &&
      echo b > file2 && git add file2 && git commit -m second --date "$GIT_AUTHOR_DATE" &&
      export GIT_AUTHOR_DATE="2020-01-03T00:00:00 +0000" &&
      export GIT_COMMITTER_DATE="$GIT_AUTHOR_DATE" &&
      echo c > file3 && git add file3 && git commit -m third --date "$GIT_AUTHOR_DATE"
    )
    export REPO_ROOT="$repo_root"
    it "ranks files by frecency" && {
      WITH_SNAPSHOT="$snapshot/default" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- frecency --working-dir \"$PWD\" --now 1578096000 2>/dev/null)"
    }
    it "supports --order ascending" && {
      WITH_SNAPSHOT="$snapshot/ascending" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- frecency --working-dir \"$PWD\" --order ascending --now 1578096000 2>/dev/null)"
    }
    it "filters paths" && {
      WITH_SNAPSHOT="$snapshot/filter" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- frecency --working-dir \"$PWD\" --paths file2 --now 1578096000 2>/dev/null)"
    }
  )
)
