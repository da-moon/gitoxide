# git-productivity-analyzer

This crate provides command line utilities to analyze developer activity in a Git repository.  
It relies on `gitoxide-core` for heavy lifting and focuses on summarizing how much time contributors have invested.

## Subcommands

- `hours` — estimate the total hours spent on the project.
  - `--working-dir` - path to the repository
  - `--rev-spec` - revision to analyze
  - `--no-bots` - ignore commits by GitHub bots
  - `--file-stats` - collect file statistics
  - `--line-stats` - collect line statistics
  - `--show-pii` - show personally identifiable information
  - `--omit-unify-identities` - don't deduplicate identities
  - `--threads <n>` - number of threads to use

All commands accept the global options `--since <date>`, `--until <date>` and `--json` to limit the date range and control the output format.

## Time Estimation Algorithm

The implementation is based on `gitoxide-core::hours::estimate_hours()` which groups commits by author and time. Commits spaced less than two hours apart are considered part of the same working session. Each session starts with an initial two hour bonus to cover context switching. Optionally the diff of each commit can be examined to track files and lines changed. Identities are unified via `.mailmap` and GitHub bots can be ignored.
