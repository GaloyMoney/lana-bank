use csv::{ReaderBuilder, Trim};
use std::io::Cursor;

use super::primitives::{AccountCategory, AccountCodeSection, AccountSpec};

use thiserror::Error;

#[derive(Error, Debug)]
#[error("CsvParseError")]
pub struct CsvParseError;

pub struct CsvParser {
    data: String,
}
impl CsvParser {
    pub fn new(data: String) -> Self {
        Self { data }
    }

    pub fn account_specs(self) -> Result<Vec<AccountSpec>, CsvParseError> {
        let mut rdr = ReaderBuilder::new()
            .flexible(true)
            .trim(Trim::All)
            .has_headers(false)
            .from_reader(Cursor::new(self.data));

        let mut specs = vec![];
        for result in rdr.records() {
            match result {
                Ok(record) => {
                    let mut sections = vec![];
                    if record.iter().all(|field| field.is_empty()) {
                        continue;
                    }

                    for field in record.iter() {
                        if let Ok(category) = field.parse::<AccountCategory>() {
                            specs.push(AccountSpec::new(sections, category));
                            break;
                        }
                        if let Ok(section) = field.parse::<AccountCodeSection>() {
                            sections.push(section);
                        }
                    }
                }
                Err(e) => eprintln!("Error reading record: {}", e),
            }
        }

        Ok(specs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_specs() {
        let data = r#"1,,,Assets"#;
        let parser = CsvParser::new(data.to_string());
        let specs = parser.account_specs().unwrap();
        assert_eq!(specs.len(), 1);
    }
}
