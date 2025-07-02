/// Compute the median of a slice of numbers.
///
/// The input slice does not need to be sorted; this function will
/// partially sort a copy internally using [`select_nth_unstable`].
pub fn median(values: &[u32]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut buf = values.to_vec();
    let mid = buf.len() / 2;
    let mid_val = *buf.select_nth_unstable(mid).1;
    if buf.len() % 2 == 1 {
        mid_val as f64
    } else {
        let lower_val = *buf[..mid].select_nth_unstable(mid - 1).1;
        (lower_val as f64 + mid_val as f64) / 2.0
    }
}

/// Return the percentile `pct` of a sorted slice of numbers.
///
/// `pct` must be within `0..=100` and `values` must be sorted in ascending order.
pub fn percentile_of_sorted(values: &[u32], pct: f64) -> Option<u32> {
    if values.is_empty() {
        return None;
    }
    assert!((0.0..=100.0).contains(&pct));
    if (pct - 100.0).abs() < f64::EPSILON || values.len() == 1 {
        return Some(values.last().copied().unwrap_or_default());
    }
    let length = (values.len() - 1) as f64;
    let rank = (pct / 100.0) * length;
    let lower = rank.floor();
    let d = rank - lower;
    let n = lower as usize;
    let lo = values[n] as f64;
    let hi = values[n + 1] as f64;
    Some((lo + (hi - lo) * d).round() as u32)
}
