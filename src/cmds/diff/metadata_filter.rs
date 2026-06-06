use super::subcmds::ValidatedThreshold;
use chrono::NaiveDate;
use log::debug;

use crate::containers::ServiceMetdata;

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

    fn filter_miles(next_service_miles: u32, miles_threshold: u32) -> bool {
        debug!(
            "Filtering where next service miles {next_service_miles} <= threshold miles {miles_threshold}"
        );
        next_service_miles <= miles_threshold
    }

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
