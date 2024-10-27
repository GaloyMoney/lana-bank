use thiserror::Error;

#[derive(Error, Debug)]
pub enum CalaError {
    #[error("CalaError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("CalaError - UnknownGqlError: {0}")]
    UnknownGqlError(String),
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
