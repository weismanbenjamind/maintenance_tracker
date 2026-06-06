//  Floor to ensure we always undershoot maintenance interval
pub(crate) fn months_to_days_floored(months: u32) -> i64 {
    (months as f64 / 12.0 * 365.0).floor() as i64
}
