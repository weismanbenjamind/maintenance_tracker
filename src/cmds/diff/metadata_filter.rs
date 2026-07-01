//! # Metadata Filter
//!
//! Houses a filtering operations for ServiceMetadata structs

use super::subcmds::ValidatedThreshold;
use chrono::NaiveDate;
use log::debug;

use crate::containers::ServiceMetdata;

/// Enum to represent a filter.
/// Can filter on miles, date, or miles/date combinations.
#[derive(Debug, Clone, Copy)]
pub(super) enum MetadataFilter {
    Miles {
        miles_threshold: u32,
    },
    Date {
        date_threshold: NaiveDate,
    },
    MilesAndDate {
        miles_threshold: u32,
        date_threshold: NaiveDate,
    },
}

impl MetadataFilter {
    /// Apply the filter.
    ///
    /// If miles filter returns true if the metadata's next service miles is below or equal to the filter's threshold.
    ///
    /// If date filter returns true if the metadata's next service date is below or equal to the filter's threshold.
    ///
    /// If miles and date filter returns true if the metadata's next service miles is below or equal to the filter's threshold
    /// or if the metadata's next service date is below or equal to the filter's threshold.
    pub(super) fn apply(&self, metadata: &ServiceMetdata) -> bool {
        let next_service = metadata.next_service();
        debug!("Filtering service {:?}", next_service);

        let result = match self {
            Self::Miles { miles_threshold } => {
                Self::filter_miles(next_service.miles(), *miles_threshold) // Keep if the next service miles is below threshold
            }
            Self::Date { date_threshold } => {
                Self::filter_date(next_service.date(), *date_threshold) // Keep if next service date is below threshold
            }
            Self::MilesAndDate {
                miles_threshold,
                date_threshold,
            } => {
                Self::filter_miles(next_service.miles(), *miles_threshold)
                    || Self::filter_date(next_service.date(), *date_threshold) // Keep if next service miles or next service date is below threshold
            }
        };
        debug!("Filter result {result}");
        result
    }

    /// Returns true if next_service_miles <= miles_threshold
    fn filter_miles(next_service_miles: u32, miles_threshold: u32) -> bool {
        debug!(
            "Filtering where next service miles {next_service_miles} <= threshold miles {miles_threshold}"
        );
        next_service_miles <= miles_threshold
    }

    /// Returns true if next_service_date <= date_threshold
    fn filter_date(next_service_date: NaiveDate, date_threshold: NaiveDate) -> bool {
        debug!(
            "Filtering where next service date {next_service_date} <= threshold date {date_threshold}"
        );
        next_service_date <= date_threshold
    }
}

impl From<ValidatedThreshold> for MetadataFilter {
    fn from(value: ValidatedThreshold) -> Self {
        match value {
            ValidatedThreshold::Miles {
                miles_threshold, ..
            } => Self::Miles { miles_threshold },
            ValidatedThreshold::Date { date_threshold } => Self::Date { date_threshold },
            ValidatedThreshold::MilesAndDate {
                miles_threshold,
                date_threshold,
                ..
            } => Self::MilesAndDate {
                miles_threshold,
                date_threshold,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::build_metadata;
    use crate::testing::constants::NEXT_SERVICE;
    use chrono::TimeDelta;

    #[test]
    fn metadata_filter_apply_miles_and_date_both_eq() {
        let filter = MetadataFilter::MilesAndDate {
            miles_threshold: NEXT_SERVICE.miles(),
            date_threshold: NEXT_SERVICE.date(),
        };
        assert!(filter.apply(&build_metadata()));
    }

    #[test]
    fn metadata_filter_apply_miles_and_date_date_fail() {
        let filter = MetadataFilter::MilesAndDate {
            miles_threshold: NEXT_SERVICE.miles() - 500,
            date_threshold: NEXT_SERVICE.date() - TimeDelta::days(1),
        };
        assert!(!filter.apply(&build_metadata()));
    }

    #[test]
    fn metadata_filter_apply_miles_and_date_date_pass() {
        let filter = MetadataFilter::MilesAndDate {
            miles_threshold: NEXT_SERVICE.miles() - 500,
            date_threshold: NEXT_SERVICE.date() + TimeDelta::days(1),
        };
        assert!(filter.apply(&build_metadata()));
    }

    #[test]
    fn metadata_filter_apply_miles_and_date_miles_pass() {
        let filter = MetadataFilter::MilesAndDate {
            miles_threshold: NEXT_SERVICE.miles() + 500,
            date_threshold: NEXT_SERVICE.date() - TimeDelta::days(1),
        };
        assert!(filter.apply(&build_metadata()));
    }

    #[test]
    fn metadata_filter_apply_date() {
        let filter = MetadataFilter::Date {
            date_threshold: NEXT_SERVICE.date() + TimeDelta::days(1),
        };
        assert!(filter.apply(&build_metadata()))
    }

    #[test]
    fn metadata_filter_apply_miles() {
        let filter = MetadataFilter::Miles {
            miles_threshold: NEXT_SERVICE.miles() + 500,
        };
        assert!(filter.apply(&build_metadata()))
    }

    #[test]
    fn metadata_filter_filter_miles() {
        assert!(MetadataFilter::filter_miles(1400, 1500));
        assert!(MetadataFilter::filter_miles(1500, 1500));
        assert!(!MetadataFilter::filter_miles(1600, 1500));
    }

    #[test]
    fn metadata_filter_filter_date() {
        let date_threshold = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
        assert!(MetadataFilter::filter_date(
            date_threshold - TimeDelta::days(1),
            date_threshold
        ));
        assert!(MetadataFilter::filter_date(date_threshold, date_threshold));
        assert!(!MetadataFilter::filter_date(
            date_threshold + TimeDelta::days(1),
            date_threshold
        ));
    }

    #[test]
    fn metdata_filter_from_validated_threshold_miles() {
        let target_miles = 1000;
        let value = ValidatedThreshold::Miles {
            miles_threshold: target_miles,
            curr_miles: 500,
        };
        let found = MetadataFilter::from(value);
        match found {
            MetadataFilter::Miles { miles_threshold } => assert_eq!(miles_threshold, target_miles),
            _ => panic!("Expected MetadataFilter::Miles"),
        }
    }

    #[test]
    fn metdata_filter_from_validated_threshold_date() {
        let target_date = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
        let value = ValidatedThreshold::Date {
            date_threshold: target_date,
        };
        let found = MetadataFilter::from(value);
        match found {
            MetadataFilter::Date { date_threshold } => assert_eq!(date_threshold, target_date),
            _ => panic!("Expected MetadataFilter::Date"),
        }
    }

    #[test]
    fn metdata_filter_from_validated_threshold_miles_and_date() {
        let target_date = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
        let target_miles = 1000;
        let value = ValidatedThreshold::MilesAndDate {
            miles_threshold: target_miles,
            curr_miles: 500,
            date_threshold: target_date,
        };
        let found = MetadataFilter::from(value);
        match found {
            MetadataFilter::MilesAndDate {
                miles_threshold,
                date_threshold,
            } => {
                assert_eq!(miles_threshold, target_miles);
                assert_eq!(date_threshold, target_date);
            }
            _ => panic!("Expected MetadataFilter::MilesAndDate"),
        }
    }
}
