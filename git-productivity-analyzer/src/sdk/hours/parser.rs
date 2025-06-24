use super::analyzer::Summary;

pub(crate) fn parse_summary(out: &str) -> Summary {
    let mut summary = Summary::default();
    for line in out.lines() {
        parse_line(line, &mut summary);
    }
    summary
}

fn parse_line(line: &str, summary: &mut Summary) {
    if let Some(v) = line.strip_prefix("total hours: ") {
        summary.total_hours = v.trim().parse().unwrap_or_default();
    } else if let Some(v) = line.strip_prefix("total 8h days: ") {
        summary.total_8h_days = v.trim().parse().unwrap_or_default();
    } else if let Some(v) = line.strip_prefix("total commits = ") {
        let num = v.split_whitespace().next().unwrap_or("0");
        summary.total_commits = num.parse().unwrap_or_default();
    } else if let Some(v) = line.strip_prefix("total authors: ") {
        summary.total_authors = v.trim().parse().unwrap_or_default();
    } else if let Some(v) = line.strip_prefix("total files added/removed/modified/remaining: ") {
        summary.total_files = parse_numbers::<4>(v);
    } else if let Some(v) = line.strip_prefix("total lines added/removed/remaining: ") {
        summary.total_lines = parse_numbers::<3>(v);
    }
}

fn parse_numbers<const N: usize>(v: &str) -> Option<[u32; N]> {
    let parts: Vec<u32> = v
        .split('/')
        .filter_map(|p| p.split_whitespace().next())
        .filter_map(|p| p.parse().ok())
        .collect();
    if parts.len() == N {
        let mut arr = [0u32; N];
        arr.copy_from_slice(&parts);
        Some(arr)
    } else {
        None
    }
}
