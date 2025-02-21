use chart_of_accounts::TransactionAccountFactory;

#[derive(Clone)]
pub struct DepositAccountFactories {
    pub deposits: TransactionAccountFactory,
    pub deposits_omnibus: TransactionAccountFactory,
}
