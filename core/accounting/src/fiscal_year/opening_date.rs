use chrono::{Datelike, NaiveDate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Jurisdiction {
    ElSalvador,
}

pub(super) struct OpeningDateForJurisdiction {
    _jurisdiction: Jurisdiction,
    date: NaiveDate,
}

impl OpeningDateForJurisdiction {
    pub fn new(date: impl Into<NaiveDate>, jurisdiction: Jurisdiction) -> Self {
        let date = date.into();

        match jurisdiction {
            Jurisdiction::ElSalvador => Self::el_salvador(date),
        }
    }

    pub fn date(&self) -> NaiveDate {
        self.date
    }

    fn el_salvador(date: NaiveDate) -> Self {
        // El Salvador fiscal year runs January 1 - December 31
        let date = NaiveDate::from_ymd_opt(date.year(), 1, 1)
            .expect("Invalid date for El Salvador fiscal year");

        Self {
            _jurisdiction: Jurisdiction::ElSalvador,
            date,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jurisdiction() {
        let opening_date = OpeningDateForJurisdiction::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            Jurisdiction::ElSalvador,
        );
        assert_eq!(opening_date._jurisdiction, Jurisdiction::ElSalvador);
    }

    #[test]
    fn el_salvador() {
        let test_cases = [
            (
                NaiveDate::from_ymd_opt(2024, 3, 10).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
            (
                NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
            (
                NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            ),
        ];

        for (input_date, expected_opening) in test_cases {
            let opening_date = OpeningDateForJurisdiction::el_salvador(input_date);
            assert_eq!(opening_date.date(), expected_opening);
        }
    }
}
