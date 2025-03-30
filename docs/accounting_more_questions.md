- what is this vec of String?
https://github.com/GaloyMoney/cala/blob/f8af9e95e7f9f987d1a91af58f894e8a033720ca/cala-ledger/src/account_set/entity.rs#L23C21-L23C27

- how is the update of the parent ledger enforced if this is passed through metadata?
https://github.com/GaloyMoney/lana-bank/blob/f6f63f38aeaa778a926af3ddbc986c610e68df7d/core/deposit/src/ledger/mod.rs#L660

this is the metadata?

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountSetValues {
    pub id: AccountSetId,
    pub version: u32,
    pub journal_id: JournalId,
    pub name: String,
    pub external_id: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>, // here?
    pub normal_balance_type: DebitOrCredit,
}




chart of account is a concept in lana

cala has the account set relationship. DAG is constructed from cala. 

anything related to account code logic is lana (using external_account_id from cala?)

"when all the checks are done we're simply attaching the account set in cala"







source of truth currently is in the credit (/ deposit (?)) module. 

the ledger is acting as an assert?