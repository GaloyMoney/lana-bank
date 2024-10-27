use std::{fmt::Display, str::FromStr};

use crate::primitives::{CustomerId, LoanId};

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum Object {
    Applicant,
    Loan(LoanAllOrOne),
    TermsTemplate,
    User,
    Customer(CustomerAllOrOne),
    Document,
    Deposit,
    Withdraw,
    Report,
    Audit,
    Ledger,
    CreditFacility,
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = ObjectDiscriminants::from(self);
        use Object::*;
        match self {
            Loan(loan_ref) => write!(f, "{}/{}", discriminant, loan_ref),
            Customer(customer_ref) => {
                write!(f, "{}/{}", discriminant, customer_ref)
            }
            _ => write!(f, "{}", discriminant),
        }
    }
}

impl FromStr for Object {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split('/');
        let entity = elems.next().expect("missing first element");
        use ObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Applicant => Object::Applicant,
            Loan => {
                let loan_ref = elems
                    .next()
                    .ok_or("could not parse Object")?
                    .parse()
                    .map_err(|_| "could not parse Object")?;
                Object::Loan(loan_ref)
            }
            TermsTemplate => Object::TermsTemplate,
            User => Object::User,
            Customer => {
                let customer_ref = elems
                    .next()
                    .ok_or("could not parse Object")?
                    .parse()
                    .map_err(|_| "could not parse Object")?;
                Object::Customer(customer_ref)
            }
            Deposit => Object::Deposit,
            Withdraw => Object::Withdraw,
            Report => Object::Report,
            Audit => Object::Audit,
            Ledger => Object::Ledger,
            CreditFacility => Object::CreditFacility,
            Document => Object::Document,
        };
        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AllOrOne<T> {
    All,
    ById(T),
}

impl<T> FromStr for AllOrOne<T>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "*" => Ok(AllOrOne::All),
            _ => {
                let id = T::from_str(s).map_err(|e| format!("Invalid ID: {}", e))?;
                Ok(AllOrOne::ById(id))
            }
        }
    }
}

impl<T> Display for AllOrOne<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AllOrOne::All => write!(f, "*"),
            AllOrOne::ById(id) => write!(f, "{}", id),
        }
    }
}

pub type LoanAllOrOne = AllOrOne<LoanId>;
pub type CustomerAllOrOne = AllOrOne<CustomerId>;
