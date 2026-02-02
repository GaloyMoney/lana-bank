use thiserror::Error;

#[derive(Error, Debug)]
pub enum CollectionsError {
    #[error("CollectionsError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CollectionsError - EsEntity: {0}")]
    EsEntity(es_entity::EsEntityError),
}

impl From<es_entity::EsEntityError> for CollectionsError {
    fn from(err: es_entity::EsEntityError) -> Self {
        Self::EsEntity(err)
    }
}
