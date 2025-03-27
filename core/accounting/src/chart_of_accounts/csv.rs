use cala_ledger::DebitOrCredit;
use csv::{ReaderBuilder, Trim};
use std::io::Cursor;

use crate::primitives::{
    AccountCodeSection, AccountCodeSectionParseError, AccountName, AccountSpec,
};

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

        let mut specs: Vec<AccountSpec> = vec![];
        for result in rdr.records() {
            match result {
                Ok(record) => {
                    let mut initial_empty = true;
                    let mut sections = vec![];
                    if record.iter().all(|field| field.is_empty()) {
                        continue;
                    }

                    let normal_balance_type = record
                        .get(4)
                        .and_then(|b| b.parse::<DebitOrCredit>().ok())
                        .unwrap_or_default();

                    for (idx, field) in record.iter().enumerate() {
                        if let Ok(category) = field.parse::<AccountName>() {
                            if let Some(s) = specs.iter().rposition(|s| s.code.is_parent(&sections))
                            {
                                specs.push(AccountSpec::new(
                                    Some(specs[s].code.clone()),
                                    sections,
                                    category,
                                    specs[s].normal_balance_type,
                                ));
                                break;
                            }
                            specs.push(AccountSpec::new(
                                None,
                                sections,
                                category,
                                normal_balance_type,
                            ));
                            break;
                        }
                        match field.parse::<AccountCodeSection>() {
                            Ok(section) => {
                                initial_empty = false;
                                sections.push(section)
                            }
                            Err(AccountCodeSectionParseError::Empty) if initial_empty => {
                                sections.push(
                                    specs
                                        .last()
                                        .expect("No parent")
                                        .code
                                        .section(idx)
                                        .expect("No parent section")
                                        .clone(),
                                );
                            }
                            _ => {
                                continue;
                            }
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
    fn parse_one_line() {
        let data = r#"1,,,Assets"#;
        let parser = CsvParser::new(data.to_string());
        let specs = parser.account_specs().unwrap();
        assert_eq!(specs.len(), 1);
    }

    #[test]
    fn parse_two_lines() {
        let data = r#"
        1,,,Assets ,Debit,
        ,,,,,
        11,,,Assets,,
        ,,,,,
        "#;
        let parser = CsvParser::new(data.to_string());
        let specs = parser.account_specs().unwrap();
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].code.len_sections(), 1);
        assert_eq!(Some(&specs[0].code), specs[1].parent.as_ref());
        assert_eq!(specs[0].normal_balance_type, DebitOrCredit::Debit);
        assert_eq!(specs[1].normal_balance_type, DebitOrCredit::Debit);
    }

    #[test]
    fn parse_child_with_empty_top_section() {
        let data = r#"
        1,,,Assets ,,
        ,,,,,
        11,,,Assets,,
        ,,,,,
            ,01,,Effective,,
        ,,0101,Central Office,
        "#;
        let parser = CsvParser::new(data.to_string());
        let specs = parser.account_specs().unwrap();
        assert_eq!(specs.len(), 4);

        assert_eq!(specs[2].code.len_sections(), 2);
        assert_eq!(Some(&specs[1].code), specs[2].parent.as_ref());

        assert_eq!(specs[3].code.len_sections(), 3);
        assert_eq!(Some(&specs[2].code), specs[3].parent.as_ref());

        assert_eq!(&specs[3].code.to_string(), "11.01.0101");
    }

    #[test]
    fn parse_when_parent_has_multiple_child_nodes() {
        let data = r#"
        1,,,Assets,,
        ,,,,
        11,,,Current Assets,,
        ,,,,,
            ,01,,Cash and Equivalents,,
        ,,0101,,Operating Cash,,
        ,,0102,,Petty Cash,,
        "#;

        let parser = CsvParser::new(data.to_string());
        let specs = parser.account_specs().unwrap();

        assert_eq!(specs.len(), 5);

        assert_eq!(specs[2].code.len_sections(), 2);
        assert_eq!(Some(&specs[1].code), specs[2].parent.as_ref());

        assert_eq!(specs[3].code.len_sections(), 3);
        assert_eq!(Some(&specs[2].code), specs[3].parent.as_ref());
        assert_eq!(&specs[3].code.to_string(), "11.01.0101");

        assert_eq!(Some(&specs[2].code), specs[4].parent.as_ref());
        assert_eq!(&specs[4].code.to_string(), "11.01.0102");
    }
}
