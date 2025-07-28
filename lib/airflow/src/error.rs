use thiserror::Error;

#[derive(Error, Debug)]
pub enum AirflowError {
    #[error("AirflowError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("AirflowError - UrlParse: {0}")]
    Url(#[from] url::ParseError),
    #[error("AirflowError - ApiError")]
    ApiError,
}
