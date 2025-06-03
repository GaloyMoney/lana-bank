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

pub type CustodianConfigAllOrOne = AllOrOne<CustodianConfigId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCustodyObject {
    CustodianConfig(CustodianConfigAllOrOne),
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum CustodianConfigAction {
    Create,
    List,
}
