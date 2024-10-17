use sqlx::PgPool;

use std::collections::HashMap;

use crate::{data_export::Export, entity::*, primitives::*};

use super::{cursor::*, entity::*, error::*};

const BQ_TABLE_NAME: &str = "customer_events";

#[derive(Clone)]
pub struct CustomerRepo {
    pool: PgPool,
    export: Export,
}

impl CustomerRepo {
    pub(super) fn new(pool: &PgPool, export: &Export) -> Self {
        Self {
            pool: pool.clone(),
            export: export.clone(),
        }
    }

    pub(super) async fn create_in_tx(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        new_customer: NewCustomer,
    ) -> Result<Customer, CustomerError> {
        sqlx::query!(
            r#"INSERT INTO customers (id, email, telegram_id)
            VALUES ($1, $2, $3)"#,
            new_customer.id as CustomerId,
            new_customer.email,
            new_customer.telegram_id,
        )
        .execute(&mut **db)
        .await?;
        let mut events = new_customer.initial_events();
        let n_events = events.persist(db).await?;
        self.export
            .export_last(db, BQ_TABLE_NAME, n_events, &events)
            .await?;
        Ok(Customer::try_from(events)?)
    }

    pub async fn find_by_id(&self, customer_id: CustomerId) -> Result<Customer, CustomerError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT a.id, e.sequence, e.event,
                a.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM customers a
            JOIN customer_events e
            ON a.id = e.id
            WHERE a.id = $1"#,
            customer_id as CustomerId
        )
        .fetch_all(&self.pool)
        .await?;
        match EntityEvents::load_first(rows) {
            Ok(customer) => Ok(customer),
            Err(EntityError::NoEntityEventsPresent) => {
                Err(CustomerError::CouldNotFindById(customer_id))
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Customer, CustomerError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT a.id, e.sequence, e.event,
                a.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM customers a
            JOIN customer_events e
            ON a.id = e.id
            WHERE a.email = $1"#,
            email
        )
        .fetch_all(&self.pool)
        .await?;
        match EntityEvents::load_first(rows) {
            Ok(customer) => Ok(customer),
            Err(EntityError::NoEntityEventsPresent) => {
                Err(CustomerError::CouldNotFindByEmail(email.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn persist_in_tx(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        customer: &mut Customer,
    ) -> Result<(), CustomerError> {
        sqlx::query!(
            r#"UPDATE customers SET telegram_id = $2 WHERE id = $1"#,
            customer.id as CustomerId,
            customer.telegram_id,
        )
        .execute(&mut **db)
        .await?;
        let n_events = customer.events.persist(db).await?;
        self.export
            .export_last(db, BQ_TABLE_NAME, n_events, &customer.events)
            .await?;
        Ok(())
    }

    pub async fn list(
        &self,
        query: crate::query::PaginatedQueryArgs<CustomerByEmailCursor>,
    ) -> Result<crate::query::PaginatedQueryRet<Customer, CustomerByEmailCursor>, CustomerError>
    {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"
            WITH customers AS (
              SELECT id, email, created_at
              FROM customers
              WHERE ((email, id) > ($2, $1)) OR ($1 IS NULL AND $2 IS NULL)
              ORDER BY email, id
              LIMIT $3
            )
            SELECT a.id, e.sequence, e.event,
                a.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM customers a
            JOIN customer_events e ON a.id = e.id
            ORDER BY a.email, a.id, e.sequence"#,
            query.after.as_ref().map(|c| c.id) as Option<CustomerId>,
            query.after.map(|c| c.email),
            query.first as i64 + 1
        )
        .fetch_all(&self.pool)
        .await?;
        let (entities, has_next_page) = EntityEvents::load_n::<Customer>(rows, query.first)?;
        let mut end_cursor = None;
        if let Some(last) = entities.last() {
            end_cursor = Some(CustomerByEmailCursor {
                id: last.id,
                email: last.email.clone(),
            });
        }
        Ok(crate::query::PaginatedQueryRet {
            entities,
            has_next_page,
            end_cursor,
        })
    }

    pub async fn find_all<T: From<Customer>>(
        &self,
        ids: &[CustomerId],
    ) -> Result<HashMap<CustomerId, T>, CustomerError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT a.id, e.sequence, e.event,
                a.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM customers a
            JOIN customer_events e
            ON a.id = e.id
            WHERE a.id = ANY($1)"#,
            ids as &[CustomerId]
        )
        .fetch_all(&self.pool)
        .await?;
        let n = rows.len();
        let res = EntityEvents::load_n::<Customer>(rows, n)?;

        Ok(res.0.into_iter().map(|u| (u.id, T::from(u))).collect())
    }
}
