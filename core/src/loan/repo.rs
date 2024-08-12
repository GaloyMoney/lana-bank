use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    entity::*,
    primitives::{CustomerId, LoanId},
};

use super::{error::LoanError, Loan, LoanCursor, NewLoan};

#[derive(Clone)]
pub struct LoanRepo {
    pool: PgPool,
}

impl LoanRepo {
    pub(super) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn create_in_tx(
        &self,
        db: &mut Transaction<'_, Postgres>,
        new_loan: NewLoan,
    ) -> Result<Loan, LoanError> {
        sqlx::query!(
            r#"INSERT INTO loans (id, customer_id)
            VALUES ($1, $2)"#,
            new_loan.id as LoanId,
            new_loan.customer_id as CustomerId,
        )
        .execute(&mut **db)
        .await?;
        let mut events = new_loan.initial_events();
        events.persist(db).await?;
        Ok(Loan::try_from(events)?)
    }

    pub async fn find_by_id(&self, id: LoanId) -> Result<Loan, LoanError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT l.id, e.sequence, e.event,
                      l.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM loans l
            JOIN loan_events e ON l.id = e.id
            WHERE l.id = $1
            ORDER BY e.sequence"#,
            id as LoanId,
        )
        .fetch_all(&self.pool)
        .await?;

        let res = EntityEvents::load_first::<Loan>(rows)?;
        Ok(res)
    }

    pub async fn persist_in_tx(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        loan: &mut Loan,
    ) -> Result<(), LoanError> {
        loan.events.persist(db).await?;
        Ok(())
    }

    pub async fn find_for_customer(&self, customer_id: CustomerId) -> Result<Vec<Loan>, LoanError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT l.id, e.sequence, e.event,
                      l.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM loans l
            JOIN loan_events e ON l.id = e.id
            WHERE l.customer_id = $1
            ORDER BY l.id, e.sequence"#,
            customer_id as CustomerId,
        )
        .fetch_all(&self.pool)
        .await?;

        let n = rows.len();
        let res = EntityEvents::load_n::<Loan>(rows, n)?;
        Ok(res.0)
    }

    pub async fn list(
        &self,
        query: crate::query::PaginatedQueryArgs<LoanCursor>,
    ) -> Result<crate::query::PaginatedQueryRet<Loan, LoanCursor>, LoanError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"
            WITH loans AS (
                SELECT id, customer_id, created_at
                FROM loans
                WHERE ($1::uuid IS NULL OR id > $1)
                ORDER BY id
                LIMIT $2
            )
            SELECT l.id, e.sequence, e.event,
                l.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM loans l
            JOIN loan_events e ON l.id = e.id
            ORDER BY l.id, e.sequence;
            "#,
            query.after.as_ref().map(|c| c.id) as Option<LoanId>,
            query.first as i64 + 1
        )
        .fetch_all(&self.pool)
        .await?;
        let (entities, has_next_page) = EntityEvents::load_n::<Loan>(rows, query.first)?;
        let mut end_cursor = None;
        if let Some(last) = entities.last() {
            end_cursor = Some(LoanCursor { id: last.id });
        }
        Ok(crate::query::PaginatedQueryRet {
            entities,
            has_next_page,
            end_cursor,
        })
    }
}
