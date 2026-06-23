//! # Dates
//!
//! Houses common logic for dealing with date operations

/// Converts months to days. Flooring the result is not a whole number.
/// Floor prevents accidentally overshooting a maintenance item.
pub(crate) fn months_to_days_floored(months: u32) -> i64 {
    //  Floor to ensure we always undershoot maintenance interval
    (months as f64 / 12.0 * 365.0).floor() as i64
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
        assert_eq!(months_to_days_floored(35), 1064)
    }
}
