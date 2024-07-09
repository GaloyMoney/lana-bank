use thiserror::Error;

#[derive(Error, Debug)]
pub enum CalaError {
    #[error("CalaError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("CalaError - UnknownGqlError: {0}")]
    UnknownGqlError(String),
    #[error("CalaError - MissingDataField")]
    MissingDataField,
    #[error("CalaError - CouldNotFindAccountByCode: {0}")]
    CouldNotFindAccountByCode(String),
}

impl From<Vec<graphql_client::Error>> for CalaError {
    fn from(errors: Vec<graphql_client::Error>) -> Self {
        let mut error_string = String::new();
        for error in errors {
            error_string.push_str(&format!("{:?}\n", error));
        }
        CalaError::UnknownGqlError(error_string)
    }
}

impl Clone for CalaError {
    fn clone(&self) -> Self {
        match self {
            CalaError::Reqwest(_) => {
                CalaError::UnknownGqlError("Reqwest error cannot be cloned".to_string())
            }
            CalaError::UnknownGqlError(err) => CalaError::UnknownGqlError(err.clone()),
            CalaError::MissingDataField => CalaError::MissingDataField,
            CalaError::CouldNotFindAccountByCode(err) => {
                CalaError::CouldNotFindAccountByCode(err.clone())
            }
        }
    }
}
