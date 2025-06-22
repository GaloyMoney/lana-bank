use es_entity::*;

use super::{entity::*, error::LoanAgreementError, primitives::*};
use crate::primitives::CustomerId;

pub mod loan_agreement_cursor {
    use es_entity::*;
    use super::*;

    cursor_pagination! {
        LoanAgreementsByCreatedAtCursor,
        LoanAgreement,
        "loan_agreements_by_created_at",
        created_at,
        chrono::DateTime<chrono::Utc>
    }
}

es_entity::es_repo! {
    LoanAgreementRepo,
    LoanAgreement,
    LoanAgreementEvent,
    LoanAgreementError,
    loan_agreements,
    loan_agreement_events
}

impl LoanAgreementRepo {
    pub async fn find_by_customer_id_by_created_at(
        &self,
        customer_id: CustomerId,
        cursor: loan_agreement_cursor::LoanAgreementsByCreatedAtCursor,
        direction: ListDirection,
    ) -> Result<CursorPaginatedList<LoanAgreement, loan_agreement_cursor::LoanAgreementsByCreatedAtCursor>, LoanAgreementError> {
        let (query, values) = cursor.build_query(
            r#"
            FROM loan_agreements la
            WHERE la.customer_id = $customer_id
            "#,
            direction,
        );

        let rows = sqlx::query_with(&query, values.bind(customer_id))
            .fetch_all(self.pool())
            .await?;

        let agreements = self.rows_to_entities(rows).await?;
        Ok(CursorPaginatedList::from_entities(agreements, direction))
    }
}