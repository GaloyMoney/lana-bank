#[derive(thiserror::Error, Debug)]
pub enum CollateralError {
    #[error("CollateralError - ManualUpdateError: Cannot update collateral with a custodian")]
    ManualUpdateError,
}

es_entity::from_es_entity_error!(CollateralError);
