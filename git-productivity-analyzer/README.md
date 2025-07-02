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
- `streaks` — longest consecutive days with a commit per author.
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
- `ownership` — summarize code ownership by directory.
  - `--working-dir` - path to the repository
  - `--rev-spec` - revision to analyze
- `--path <glob>` - only consider paths matching this pattern
- `--author <pattern>` - filter commits by author
  - `--depth <n>` - number of path segments to group by
  - `--json` - machine readable output

All commands accept the global options `--since <date>`, `--until <date>`, `--json`, and `--log-level <level>` to limit the date range, choose output format, and control verbosity.

## Usage Examples

### hours

```bash
git-productivity-analyzer hours --working-dir path/to/repo
```

Totals show estimated hours and approximate work days based on commit spacing.

### commit-frequency

```bash
git-productivity-analyzer commit-frequency --working-dir path/to/repo
```

Counts commits per day and week to gauge sustained engagement.

### streaks

```bash
git-productivity-analyzer streaks --working-dir path/to/repo --author Alice
```

Reports the longest span of consecutive commit days for each author.

### time-of-day

```bash
git-productivity-analyzer time-of-day --working-dir path/to/repo --bins 12
```

Builds a histogram of commit times to reveal typical working hours.

### churn

```bash
git-productivity-analyzer churn --working-dir path/to/repo --per-file
```

Summarizes lines added and removed which can highlight refactoring activity.

### commit-size

```bash
git-productivity-analyzer commit-size --working-dir path/to/repo --percentiles 50,90
```

Shows how many files and lines change per commit. Large outliers may need more review.

### frecency

```bash
git-productivity-analyzer frecency --working-dir path/to/repo --path-only
```

Ranks files by a score that favors recent and frequent changes.

### ownership

```bash
git-productivity-analyzer ownership --working-dir path/to/repo --depth 1
```

Displays commit percentages per directory to identify subject matter experts.

## Time Estimation Algorithm

The implementation is based on `gitoxide-core::hours::estimate_hours()` which groups commits by author and time. Commits spaced less than two hours apart are considered part of the same working session. Each session starts with an initial two hour bonus to cover context switching. Optionally the diff of each commit can be examined to track files and lines changed. Identities are unified via `.mailmap` and GitHub bots can be ignored.

## Commit Frequency & Developer Engagement

Commit frequency helps gauge how busy contributors are and how engaged they remain over time. Regular commits across many days indicate an active developer whereas sparse contributions may show less involvement. Weekly totals can highlight periods of intense activity or lulls.
Analyzing the commit time of day reveals when individuals typically work, helping to infer personal or team schedules and preferred collaboration windows.
Streak lengths show how consistently someone contributes. Very long streaks may indicate dedication but could also hint at burnout risk when breaks are rare.

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

When percentiles are requested as well:

```json
{
  "min_files": 1,
  "max_files": 1,
  "avg_files": 1.0,
  "median_files": 1.0,
  "min_lines": 1,
  "max_lines": 3,
  "avg_lines": 2.0,
  "median_lines": 2.0,
  "line_percentiles": [
    [50.0, 2],
    [100.0, 3]
  ]
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

## Code Ownership

`ownership` shows what percentage of commits each contributor made per directory. The `--depth` option controls how many path segments are used when grouping files. Using `--depth 0` groups all files together. Files in the repository root are always grouped under the `.` directory.
Merge commits are compared against their first parent only. Changes are counted per file and aggregated by directory. Authors are sorted by descending percentage with alphabetical ordering used to break ties.
This helps identify experts for specific modules and highlights areas with a high bus factor.

## Running Tests

Execute all end-to-end checks with:

```bash
cargo test-short -p git-productivity-analyzer
```

The `test-short` and `check-short` commands are convenience aliases defined in
`.cargo/config.toml` to run `cargo test --message-format short` and
`cargo check --message-format short` respectively.
You can run `cargo check-short -p git-productivity-analyzer` for a quick build
check without executing tests.

