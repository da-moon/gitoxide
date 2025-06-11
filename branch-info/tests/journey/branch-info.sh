# Must be sourced into the main journey test

snapshot="$root/../branch-info/tests/snapshots/branch-info"

(
  title "branch-info"
  (when "running in a basic repo"
    (sandbox
      gix/tests/fixtures/make_basic_repo.sh >/dev/null
      git remote add origin bare.git
      git push origin main:main >/dev/null
      git branch --set-upstream-to=origin/main
      echo hi >> this
      git commit -am c3 >/dev/null
      it "prints current and upstream branch" && {
        WITH_SNAPSHOT="$snapshot/basic" \
        expect_run_sh $SUCCESSFULLY "cargo run -p branch-info -- --repo ."
      }
    )
  )
)
