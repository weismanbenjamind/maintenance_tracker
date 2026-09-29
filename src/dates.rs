//! # Dates
//!
//! Houses common logic for dealing with date operations

const DAYS_PER_YEAR: f64 = 365.25;

/// Converts months to days. Flooring the result is not a whole number.
/// Assumes 365.25 days/year
/// Floor prevents accidentally overshooting a maintenance item.
pub(crate) fn months_to_days_floored(months: u32) -> i64 {
    //  Floor to ensure we always undershoot maintenance interval
    (months as f64 / 12.0 * DAYS_PER_YEAR).floor() as i64
}

/// Converts a number of days to months.
///
/// Assumes 365.25 days/year.
///
/// Rounds to hundreths of a month.
pub(crate) fn days_to_months(days: i64) -> f64 {
    let exact_months = days as f64 / DAYS_PER_YEAR * 12.0;

    // Below rounds to hundreths
    (exact_months * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_months_to_days_floored_no_round() {
        assert_eq!(months_to_days_floored(36), 1095)
    }

    #[test]
    fn test_months_to_days_floored_round() {
        assert_eq!(months_to_days_floored(35), 1065)
    }

    #[test]
    fn test_days_to_months_no_round() {
        // 1461 days is 4 years
        assert_eq!(days_to_months(1461), 48.0);
    }

    #[test]
    fn test_days_to_months_round() {
        assert_eq!(days_to_months(45), 1.48);
    }
}
