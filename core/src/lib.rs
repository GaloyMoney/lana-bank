#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod app;
pub mod applicant;
pub mod audit;
pub mod authorization;
pub mod cli;
pub mod constants;
pub mod credit_facility;
pub mod customer;
pub mod data_export;
pub mod deposit;
pub mod entity;
pub mod job;
pub mod ledger;
pub mod loan;
pub mod price;
pub mod primitives;
pub mod report;
pub mod server;
pub mod terms_template;
pub mod user;
pub mod withdraw;

pub mod query {
    #[derive(Debug)]
    pub struct PaginatedQueryArgs<T: std::fmt::Debug> {
        pub first: usize,
        pub after: Option<T>,
    }

    impl<T: std::fmt::Debug> Default for PaginatedQueryArgs<T> {
        fn default() -> Self {
            Self {
                first: 100,
                after: None,
            }
        }
    }

    pub struct PaginatedQueryRet<T, C> {
        pub entities: Vec<T>,
        pub has_next_page: bool,
        pub end_cursor: Option<C>,
    }
}
