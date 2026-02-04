use cala_ledger::DebitOrCredit;
use csv::{ReaderBuilder, Trim};
use std::io::Cursor;

use crate::primitives::{
    AccountCodeError, AccountCodeSection, AccountCodeSectionParseError, AccountName, AccountSpec,
};

use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CsvParseError {
    #[error("CsvParseError - AccountCodeError")]
    AccountCodeError(#[from] AccountCodeError),
}

impl ErrorSeverity for CsvParseError {
    fn severity(&self) -> Level {
        match self {
            Self::AccountCodeError(e) => e.severity(),
        }
    }
}

pub struct CsvParser {
    import_data: String,
}
impl CsvParser {
    pub fn new(import_data: String) -> Self {
        Self { import_data }
    }

    pub fn account_specs(self) -> Result<Vec<AccountSpec>, CsvParseError> {
        let mut rdr = ReaderBuilder::new()
            .flexible(true)
            .trim(Trim::All)
            .has_headers(false)
            .from_reader(Cursor::new(self.import_data));

        let mut specs: Vec<AccountSpec> = vec![];
        for result in rdr.records() {
            match result {
                Ok(record) => {
                    let mut initial_empty = true;
                    let mut sections = vec![];
                    let mut category = None;
                    let mut normal_balance_type = None;
                    if record.iter().all(|field| field.is_empty()) {
                        continue;
                    }

                    for (idx, field) in record.iter().enumerate() {
                        if let Ok(balance_type) = field.parse::<DebitOrCredit>() {
                            normal_balance_type = Some(balance_type);
                            break;
                        }

                        if category.is_some() {
                            continue;
                        } else if let Ok(account_category) = field.parse::<AccountName>() {
                            category = Some(account_category);
                            continue;
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

                    if let Some(category) = category {
                        if let Some(parent_spec) = specs
                            .iter()
                            .rposition(|spec| spec.code.is_parent_of(&sections))
                            .map(|parent_idx| &specs[parent_idx])
                        {
                            specs.push(AccountSpec::try_new(
                                Some(parent_spec.code.clone()),
                                sections,
                                category,
                                parent_spec.normal_balance_type,
                            )?);
                        } else {
                            specs.push(AccountSpec::try_new(
                                None,
                                sections,
                                category,
                                normal_balance_type.unwrap_or_default(),
                            )?);
                        }
                    }
                }
                Err(e) => tracing::error!(error = %e, "Error reading CSV record"),
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
    fn parse_multiple_lines_and_switch_balance_type() {
        let data = r#"
        1,,,Assets ,Debit,
        ,,,,,
        11,,,Assets,,
        ,,,,,
        2,,,Liabilities ,,
        ,,,,,
        21,,,Liabilities,,
        ,,,,,
        3,,,Expenses,,,,,Debit,
        ,,,,,
        4,,,Revenues,,,,,Credit,
        ,,,,,
        "#;
        let parser = CsvParser::new(data.to_string());
        let specs = parser.account_specs().unwrap();
        assert_eq!(specs.len(), 6);

        let assets_spec = &specs[0];
        assert_eq!(assets_spec.code.len_sections(), 1);
        assert_eq!(Some(&assets_spec.code), specs[1].parent.as_ref());
        assert_eq!(assets_spec.normal_balance_type, DebitOrCredit::Debit);

        let sub_assets_spec = &specs[1];
        assert_eq!(sub_assets_spec.normal_balance_type, DebitOrCredit::Debit);

        let liabilities_spec = &specs[2];
        assert_eq!(liabilities_spec.normal_balance_type, DebitOrCredit::Credit);

        let sub_liabilities_spec = &specs[3];
        assert_eq!(
            sub_liabilities_spec.normal_balance_type,
            DebitOrCredit::Credit
        );

        let expenses_spec = &specs[4];
        assert_eq!(expenses_spec.normal_balance_type, DebitOrCredit::Debit);

        let revenues_spec = &specs[5];
        assert_eq!(revenues_spec.normal_balance_type, DebitOrCredit::Credit);
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

    #[test]
    fn parse_identifies_top_level_child_nodes() {
        let data = r#"
        1,,,Assets ,Debit,
        ,,,,,
        11,,,Current Assets,,
        ,,,,,
        "#;
        let parser = CsvParser::new(data.to_string());
        let specs = parser.account_specs().unwrap();

        let current_assets_spec = &specs[1];
        assert_eq!(Some(&specs[0].code), current_assets_spec.parent.as_ref());
        assert_eq!(
            current_assets_spec.normal_balance_type,
            DebitOrCredit::Debit
        );
    }
}
