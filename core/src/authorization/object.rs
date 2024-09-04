#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Object {
    Applicant,
    Loan,
    Term,
    User,
    Customer,
    Deposit,
    Withdraw,
    Audit,
    Ledger,
}
