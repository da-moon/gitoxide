# Must be sourced into the main journey test

title "branch-info"

snapshot="$snapshot/branch-info"

(when "listing commits and showing diff"
  repo_root="$PWD"
  (sandbox
    export REPO_ROOT="$repo_root"
    git init
    git checkout -b main
    git config commit.gpgsign false
    git config tag.gpgsign false
    echo base > file
    git add file
    git commit -m base
    git remote add origin .
    git update-ref refs/remotes/origin/main HEAD
    git checkout -b feature
    git branch --set-upstream-to=origin/main
    echo line1 >> file
    git add file
    git commit -m f1
    echo line2 >> file
    git add file
    git commit -m f2

    it "matches git log output" && {
      expect_run_sh $SUCCESSFULLY "
        (cd \"$REPO_ROOT\" && cargo run-short -p branch-info -- --repo \"$PWD\") >actual &&
        { echo 'branch: $(git branch --show-current)' && git log --format='%H %s' \$(git merge-base HEAD @{upstream})..HEAD; } >expected &&
        diff -u expected actual
      "
    }

    it "produces a diff comparable to git diff" && {
      expect_run_sh $SUCCESSFULLY "
        (cd \"$REPO_ROOT\" && cargo run-short -p branch-info -- --repo \"$PWD\" --show-diff) >actual_diff &&
        git diff \$(git merge-base HEAD @{upstream})..HEAD |
          grep -vE '^(index|--- |\\+\\+\\+ )' |
          sed 's/@@ -1[0-9,]* +1,3 @@/@@ -1,1 +1,3 @@/' >expected_diff &&
        tail -n +3 actual_diff >actual_patch &&
        diff -u expected_diff actual_patch
      "
    }
  )
)
