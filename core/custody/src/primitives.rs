use authz::AllOrOne;

es_entity::entity_id! {
    CustodianConfigId;
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCustodyAction {
    CustodianConfig(CustodianConfigAction),
}

impl CoreCustodyAction {
    pub const CUSTODIAN_CONFIG_CREATE: Self =
        CoreCustodyAction::CustodianConfig(CustodianConfigAction::Create);
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum CustodianConfigAction {
    Create,
    List,
}

pub type CustodianConfigAllOrOne = AllOrOne<CustodianConfigId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCustodyObject {
    CustodianConfig(CustodianConfigAllOrOne),
}

impl CoreCustodyObject {
    pub const fn all_custodian_configs() -> Self {
        CoreCustodyObject::CustodianConfig(AllOrOne::All)
    }
}
