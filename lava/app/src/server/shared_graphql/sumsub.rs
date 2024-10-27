use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct SumsubTokenCreatePayload {
    pub token: String,
}

#[derive(SimpleObject)]
pub struct SumsubPermalinkCreatePayload {
    pub url: String,
}
