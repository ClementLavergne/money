//! Filtering option which allows or not an `Order` according to its *date*.
pub use chrono::NaiveDate;
use NaiveDateFilter::{Between, DateIgnored, Since, Until};

/// Regroups a pair of optional `NaiveDate`.
pub struct OptionNaiveDateRange(pub Option<NaiveDate>, pub Option<NaiveDate>);

/// References different states for a date range.
#[derive(PartialEq, Debug)]
pub enum NaiveDateFilter {
    /// No date filtering is enabled.
    DateIgnored,
    /// Filtering enabled from a given date to last date.
    Since(NaiveDate),
    /// Filtering enabled from first date to a given date.
    Until(NaiveDate),
    /// Filtering enabled from a given date to another one.
    Between(NaiveDate, NaiveDate),
}

impl NaiveDateFilter {
    /// Updates value from a range of optional `NaiveDate` data.
    pub fn set_range(&mut self, range: OptionNaiveDateRange) {
        *self = match range {
            OptionNaiveDateRange(None, None) => DateIgnored,
            OptionNaiveDateRange(Some(begin), None) => Since(begin),
            OptionNaiveDateRange(None, Some(end)) => Until(end),
            OptionNaiveDateRange(Some(begin), Some(end)) => {
                NaiveDateFilter::check_range(begin, end)
            }
        }
    }

    /// Updates the start boundary only.
    pub fn set_beginning(&mut self, start_date: Option<NaiveDate>) {
        if let Some(date) = start_date {
            *self = match *self {
                DateIgnored => Since(date),
                Since(_) => Since(date),
                Until(end) | Between(_, end) => NaiveDateFilter::check_range(date, end),
            }
        } else {
            *self = match *self {
                DateIgnored | Since(_) => DateIgnored,
                Until(end) | Between(_, end) => Until(end),
            }
        }
    }

    /// Updates the end boundary only.
    pub fn set_end(&mut self, end_date: Option<NaiveDate>) {
        if let Some(date) = end_date {
            *self = match *self {
                DateIgnored => Until(date),
                Since(begin) | Between(begin, _) => NaiveDateFilter::check_range(begin, date),
                Until(_) => Until(date),
            }
        } else {
            *self = match *self {
                DateIgnored | Until(_) => DateIgnored,
                Since(begin) | Between(begin, _) => Since(begin),
            }
        }
    }

    #[inline]
    fn check_range(start_date: NaiveDate, end_date: NaiveDate) -> NaiveDateFilter {
        if end_date.signed_duration_since(start_date).num_days() >= 0 {
            Between(start_date, end_date)
        } else {
            Since(start_date)
        }
    }

    // Evaluates if a date is allowed or not.
    pub fn is_date_allowed(&self, date: Option<NaiveDate>) -> bool {
        match self {
            DateIgnored => true,
            _ if date == None => false,
            Until(end) => end.signed_duration_since(date.unwrap()).num_days() >= 0,
            Since(start) => date.unwrap().signed_duration_since(*start).num_days() >= 0,
            Between(start, end) => {
                let date = date.unwrap();
                date.signed_duration_since(*start).num_days() >= 0
                    && end.signed_duration_since(date).num_days() >= 0
            }
        }
    }
}

impl From<OptionNaiveDateRange> for NaiveDateFilter {
    fn from(range: OptionNaiveDateRange) -> Self {
        let mut filter = DateIgnored;
        filter.set_range(range);
        filter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disable() {
        let start = NaiveDate::from_ymd(2020, 2, 3);
        let end = NaiveDate::from_ymd(2020, 11, 10);

        // set_range()
        let mut date_filter = Between(start, end);
        date_filter.set_range(OptionNaiveDateRange(None, None));

        assert_eq!(date_filter, DateIgnored);

        // set_beginning()
        let mut date_filter = Since(start);
        date_filter.set_beginning(None);

        assert_eq!(date_filter, DateIgnored);

        let mut date_filter = Between(start, end);
        date_filter.set_beginning(None);

        assert_eq!(date_filter, Until(end));

        // set_end()
        let mut date_filter = Until(end);
        date_filter.set_end(None);

        assert_eq!(date_filter, DateIgnored);

        let mut date_filter = Between(start, end);
        date_filter.set_end(None);

        assert_eq!(date_filter, Since(start));
    }

    #[test]
    fn enable_from_start_date() {
        let valid_start_1 = NaiveDate::from_ymd(2020, 2, 3);
        let valid_start_2 = NaiveDate::from_ymd(2019, 5, 30);

        // set_range()
        let mut date_filter = DateIgnored;
        date_filter.set_range(OptionNaiveDateRange(Some(valid_start_1), None));

        assert_eq!(date_filter, Since(valid_start_1));

        date_filter.set_range(OptionNaiveDateRange(Some(valid_start_2), None));

        assert_eq!(date_filter, Since(valid_start_2));

        // set_beginning()
        let mut date_filter = DateIgnored;
        date_filter.set_beginning(Some(valid_start_1));

        assert_eq!(date_filter, Since(valid_start_1));

        date_filter.set_beginning(Some(valid_start_2));

        assert_eq!(date_filter, Since(valid_start_2));
    }

    #[test]
    fn enable_to_end_date() {
        let valid_end_1 = NaiveDate::from_ymd(2020, 2, 3);
        let valid_end_2 = NaiveDate::from_ymd(2019, 5, 30);

        // set_range()
        let mut date_filter = DateIgnored;
        date_filter.set_range(OptionNaiveDateRange(None, Some(valid_end_1)));

        assert_eq!(date_filter, Until(valid_end_1));

        date_filter.set_range(OptionNaiveDateRange(None, Some(valid_end_2)));

        assert_eq!(date_filter, Until(valid_end_2));

        // set_end()
        let mut date_filter = DateIgnored;
        date_filter.set_end(Some(valid_end_1));

        assert_eq!(date_filter, Until(valid_end_1));

        date_filter.set_end(Some(valid_end_2));

        assert_eq!(date_filter, Until(valid_end_2));
    }

    #[test]
    fn enable_date_range() {
        // set_range()
        let valid_start = NaiveDate::from_ymd(2020, 2, 3);
        let valid_end = NaiveDate::from_ymd(2020, 5, 5);
        let mut date_filter = DateIgnored;
        date_filter.set_range(OptionNaiveDateRange(Some(valid_start), Some(valid_end)));

        assert_eq!(date_filter, Between(valid_start, valid_end));

        // set_beginning()
        let mut date_filter = Until(valid_end);
        date_filter.set_beginning(Some(valid_start));

        assert_eq!(date_filter, Between(valid_start, valid_end));

        // set_end()
        let mut date_filter = Since(valid_start);
        date_filter.set_end(Some(valid_end));

        assert_eq!(date_filter, Between(valid_start, valid_end));
    }

    #[test]
    fn manage_invalid_date_range() {
        let valid_start = NaiveDate::from_ymd(2020, 2, 3);
        let invalid_end = NaiveDate::from_ymd(2019, 11, 10);
        let valid_end = NaiveDate::from_ymd(2020, 5, 5);
        let invalid_start = NaiveDate::from_ymd(2021, 6, 23);

        // set_range()
        let mut date_filter = DateIgnored;
        date_filter.set_range(OptionNaiveDateRange(Some(valid_start), Some(invalid_end)));

        assert_eq!(date_filter, Since(valid_start));

        let mut date_filter = DateIgnored;
        date_filter.set_range(OptionNaiveDateRange(Some(invalid_start), Some(valid_end)));

        assert_eq!(date_filter, Since(invalid_start));

        // set_beginning()
        let mut date_filter = Between(valid_start, valid_end);
        date_filter.set_beginning(Some(invalid_start));

        assert_eq!(date_filter, Since(invalid_start));

        let mut date_filter = Until(valid_end);
        date_filter.set_beginning(Some(invalid_start));

        assert_eq!(date_filter, Since(invalid_start));

        // set_end()
        let mut date_filter = Between(valid_start, valid_end);
        date_filter.set_end(Some(invalid_end));

        assert_eq!(date_filter, Since(valid_start));

        let mut date_filter = Since(valid_start);
        date_filter.set_end(Some(invalid_end));

        assert_eq!(date_filter, Since(valid_start));
    }

    #[test]
    fn allow_date() {
        let valid_start = NaiveDate::from_ymd(2020, 2, 3);
        let valid_end = NaiveDate::from_ymd(2020, 5, 5);
        let date_filter_1 = DateIgnored;
        let date_filter_2 = Since(valid_start);
        let date_filter_3 = Until(valid_end);
        let date_filter_4 = Between(valid_start, valid_end);
        let date_1 = Some(NaiveDate::from_ymd(2020, 4, 30));
        let date_2: Option<NaiveDate> = None;

        assert_eq!(date_filter_1.is_date_allowed(date_1), true);
        assert_eq!(date_filter_2.is_date_allowed(date_1), true);
        assert_eq!(date_filter_3.is_date_allowed(date_1), true);
        assert_eq!(date_filter_4.is_date_allowed(date_1), true);
        assert_eq!(date_filter_1.is_date_allowed(date_2), true);
    }

    #[test]
    fn reject_date() {
        let valid_start = NaiveDate::from_ymd(2020, 2, 3);
        let valid_end_1 = NaiveDate::from_ymd(2018, 5, 5);
        let valid_end_2 = NaiveDate::from_ymd(2020, 5, 5);
        let date_filter_1 = Since(valid_start);
        let date_filter_2 = Until(valid_end_1);
        let date_filter_3 = Between(valid_start, valid_end_2);
        let date_1 = Some(NaiveDate::from_ymd(2019, 4, 30));
        let date_2: Option<NaiveDate> = None;

        assert_eq!(date_filter_1.is_date_allowed(date_1), false);
        assert_eq!(date_filter_2.is_date_allowed(date_1), false);
        assert_eq!(date_filter_3.is_date_allowed(date_1), false);
        assert_eq!(date_filter_1.is_date_allowed(date_2), false);
        assert_eq!(date_filter_2.is_date_allowed(date_2), false);
        assert_eq!(date_filter_3.is_date_allowed(date_2), false);
    }
}
