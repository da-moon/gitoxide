#!/bin/bash
set -eu -o pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"
source "$SCRIPT_DIR/utilities.sh"
SUCCESSFULLY=0

snapshot="$SCRIPT_DIR/snapshots/code-ownership"

(title "code-ownership" && \
  repo_root="$PWD" && \
  (
    sandbox && (
      git init &&
      git checkout -b main &&
      git config commit.gpgsign false &&
      git config tag.gpgsign false &&
      mkdir src docs &&
      echo root > README && git add README && \
      GIT_AUTHOR_NAME="Alice" GIT_AUTHOR_EMAIL=a@example.com \
      GIT_COMMITTER_NAME="Alice" GIT_COMMITTER_EMAIL=a@example.com git commit -m root &&
      echo a > src/a && git add src/a && \
      GIT_AUTHOR_NAME="Alice" GIT_AUTHOR_EMAIL=a@example.com \
      GIT_COMMITTER_NAME="Alice" GIT_COMMITTER_EMAIL=a@example.com git commit -m init &&
      echo b > docs/b && git add docs/b && \
      GIT_AUTHOR_NAME="Bob" GIT_AUTHOR_EMAIL=b@example.com \
      GIT_COMMITTER_NAME="Bob" GIT_COMMITTER_EMAIL=b@example.com git commit -m docs &&
      echo more >> src/a && git add src/a && \
      GIT_AUTHOR_NAME="Alice" GIT_AUTHOR_EMAIL=a@example.com \
      GIT_COMMITTER_NAME="Alice" GIT_COMMITTER_EMAIL=a@example.com git commit -m change
    )
    export REPO_ROOT="$repo_root"
    it "prints ownership percentages" && {
      WITH_SNAPSHOT="$snapshot/default" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- ownership --working-dir \"$PWD\" 2>/dev/null)"
    }
    it "prints ownership in JSON" && {
      WITH_SNAPSHOT="$snapshot/json" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- --json ownership --working-dir \"$PWD\" 2>/dev/null)"
    }
    it "filters by path" && {
      WITH_SNAPSHOT="$snapshot/path" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- ownership --working-dir \"$PWD\" --path 'src/*' 2>/dev/null)"
    }
  it "filters by author" && {
    WITH_SNAPSHOT="$snapshot/author" \
    expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- ownership --working-dir \"$PWD\" --author Alice 2>/dev/null)"
  }
  )
)

(title "code-ownership-root-only" && \
  repo_root="$PWD" && \
  (
    sandbox && (
      git init &&
      git checkout -b main &&
      git config commit.gpgsign false &&
      git config tag.gpgsign false &&
      echo one > A && git add A && \
      GIT_AUTHOR_NAME="Alice" GIT_AUTHOR_EMAIL=a@example.com \
      GIT_COMMITTER_NAME="Alice" GIT_COMMITTER_EMAIL=a@example.com git commit -m A &&
      echo two > B && git add B && \
      GIT_AUTHOR_NAME="Bob" GIT_AUTHOR_EMAIL=b@example.com \
      GIT_COMMITTER_NAME="Bob" GIT_COMMITTER_EMAIL=b@example.com git commit -m B
    )
    export REPO_ROOT="$repo_root"
    it "groups root files and orders authors" && {
      WITH_SNAPSHOT="$snapshot/root-only" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- ownership --working-dir \"$PWD\" 2>/dev/null)"
    }
  )
)

(title "code-ownership-no-matches" && \
  repo_root="$PWD" && \
  (
    sandbox && (
      git init &&
      git checkout -b main &&
      git config commit.gpgsign false &&
      git config tag.gpgsign false &&
      mkdir src && echo hi > src/a && git add src/a && \
      GIT_AUTHOR_NAME="Alice" GIT_AUTHOR_EMAIL=a@example.com \
      GIT_COMMITTER_NAME="Alice" GIT_COMMITTER_EMAIL=a@example.com git commit -m init
    )
    export REPO_ROOT="$repo_root"
    it "handles path filter with no matches" && {
      WITH_SNAPSHOT="$snapshot/no-path" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- ownership --working-dir \"$PWD\" --path 'none*' 2>/dev/null)"
    }
    it "handles author filter with no matches" && {
      WITH_SNAPSHOT="$snapshot/no-author" \
      expect_run_sh $SUCCESSFULLY "(cd \"$REPO_ROOT\" && cargo run-short -p git-productivity-analyzer -- ownership --working-dir \"$PWD\" --author Carol 2>/dev/null)"
    }
  )
)

(title "code-ownership-empty" && \
  repo_root="$PWD" && \
  (
    sandbox && (
      git init &&
      git checkout -b main &&
      git config commit.gpgsign false &&
      git config tag.gpgsign false
    )
    export REPO_ROOT="$repo_root"
    it "fails on empty repository" && {
      WITH_SNAPSHOT="$snapshot/empty" \
      SNAPSHOT_FILTER="$SCRIPT_DIR/filters/last-line.sh" \
      expect_run_sh 1 "(cd \"$REPO_ROOT\" && cargo run-short -q -p git-productivity-analyzer -- ownership --working-dir \"$PWD\" 2>&1)"
    }
  )
)
