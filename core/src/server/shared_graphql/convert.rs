pub trait ToGlobalId {
    fn to_global_id(&self) -> async_graphql::types::ID;
}
