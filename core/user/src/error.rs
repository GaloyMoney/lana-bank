use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreUserError {
    #[error("CoreUserError - UserError: {0}")]
    UserError(#[from] super::user::UserError),
}
