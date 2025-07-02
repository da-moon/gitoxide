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
- `commit-frequency` — count commits per day and week and report active days per author.
  - `--working-dir` - path to the repository
  - `--rev-spec` - revision to analyze
  - `--author <pattern>` - filter commits by author
- `time-of-day` — show a histogram of commit times across a 24h day.
  - `--working-dir` - path to the repository
  - `--rev-spec` - revision to analyze
  - `--bins <n>` - number of bins for the histogram (1-24)
  - `--author <pattern>` - filter commits by author
- `churn` — summarize lines added and removed over time.
  - `--working-dir` - path to the repository
  - `--rev-spec` - revision to analyze
  - `--per-file` - show totals per file path
  - `--author <pattern>` - filter commits by author
- `commit-size` — summarize how many files and lines change per commit.
  - `--working-dir` - path to the repository
  - `--rev-spec` - revision to analyze
  - `--percentiles <list>` - show additional percentiles for commit size
  - `--json` - machine readable output
- `frecency` — rank files by how recently and frequently they changed.
  - `--working-dir` - path to the repository
  - `--rev-spec` - revision to analyze
  - `--paths <path>...` - only consider these paths
  - `--max-commits <n>` - limit the number of commits scanned
  - `--order <ascending|descending>` - sort results
  - `--now <secs>` - evaluate frecency at this timestamp
  - `--age-exp <n>` - exponent for age weighting
  - `--size-ref <bytes>` - reference size for the file penalty
- `--path-only` - print only file paths
- merge commits are skipped when tallying file scores
 - when multiple files have the same score they are sorted alphabetically to keep output stable

All commands accept the global options `--since <date>`, `--until <date>`, `--json`, and `--log-level <level>` to limit the date range, choose output format, and control verbosity.

## Time Estimation Algorithm

The implementation is based on `gitoxide-core::hours::estimate_hours()` which groups commits by author and time. Commits spaced less than two hours apart are considered part of the same working session. Each session starts with an initial two hour bonus to cover context switching. Optionally the diff of each commit can be examined to track files and lines changed. Identities are unified via `.mailmap` and GitHub bots can be ignored.

## Commit Frequency & Developer Engagement

Commit frequency helps gauge how busy contributors are and how engaged they remain over time. Regular commits across many days indicate an active developer whereas sparse contributions may show less involvement. Weekly totals can highlight periods of intense activity or lulls.
Analyzing the commit time of day reveals when individuals typically work, helping to infer personal or team schedules and preferred collaboration windows.

## Code Churn & Refactoring Insight

Churn measures how many lines are added and removed within a period. Frequent churn
can signal ongoing refactoring efforts, hotspots that change often, or general
development activity across the repository.

## Commit Size & Review Effort

Large commits are harder to review and carry a higher risk of introducing
problems. Keeping commit sizes small makes code reviews faster and helps isolate
issues. Small changes let reviewers focus on the intent of each commit which
reduces the chances of bugs slipping through. The `commit-size` command
summarizes how much code changes per commit so you can gauge the typical review
burden and spot unusually large commits.

When the `--json` flag is used the output looks like this:

```json
{
  "min_files": 1,
  "max_files": 1,
  "avg_files": 1.0,
  "median_files": 1.0,
  "min_lines": 1,
  "max_lines": 3,
  "avg_lines": 2.0,
  "median_lines": 2.0
}
```

## File Frecency

`frecency` ranks files by combining the age of commits touching them with the
size of each change. Every commit contributes a score of
`size_penalty(blob_size) * age_weight(days_since_commit)` to the affected files.
Recent commits therefore have a greater impact while large files are penalized.
The `--age-exp` and `--size-ref` flags allow tuning the decay rate and size penalty.
Results are printed as a list of `{path, score}` objects when `--json` is used,
while merge commits are ignored entirely. Sorting the output reveals hotspots
that changed often in the analyzed range.
Entries with identical scores are ordered alphabetically so results are
stable across runs.
Warnings encountered during analysis are emitted using the `log` crate.
Configure `RUST_LOG` or the `--log-level` flag to control their visibility.
