use anyhow::{Result, bail};
use chrono::{DateTime, NaiveDate};

pub fn normalize_graphql_date(input: &str) -> Result<String> {
    let input = input.trim();

    if let Ok(date) = NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        return Ok(date.format("%Y-%m-%d").to_string());
    }

    if let Ok(date_time) = DateTime::parse_from_rfc3339(input) {
        return Ok(date_time.date_naive().format("%Y-%m-%d").to_string());
    }

    bail!("Invalid date '{input}'. Expected YYYY-MM-DD (GraphQL Date) or RFC3339 datetime")
}

#[cfg(test)]
mod tests {
    use super::normalize_graphql_date;

    #[test]
    fn keeps_date_format() {
        let normalized = normalize_graphql_date("2026-01-01").unwrap();
        assert_eq!(normalized, "2026-01-01");
    }

    #[test]
    fn normalizes_rfc3339_to_date() {
        let normalized = normalize_graphql_date("2026-01-01T00:00:00Z").unwrap();
        assert_eq!(normalized, "2026-01-01");
    }

    #[test]
    fn rejects_invalid_date() {
        let err = normalize_graphql_date("2026/01/01").expect_err("should fail");
        assert!(
            err.to_string()
                .contains("Expected YYYY-MM-DD (GraphQL Date) or RFC3339 datetime")
        );
    }
}
