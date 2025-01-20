use crate::accounting_init::{constants::*, *};

pub(crate) async fn init(cala: &CalaLedger) -> Result<JournalInit, AccountingInitError> {
    use cala_ledger::journal::*;

    let new_journal = NewJournal::builder()
        .id(JournalId::new())
        .name("General Ledger")
        .description("General ledger for Lana")
        .code(LANA_JOURNAL_CODE)
        .build()
        .expect("new journal");

    match cala.journals().create(new_journal).await {
        Err(cala_ledger::journal::error::JournalError::CodeAlreadyExists) => {
            let journal = cala
                .journals()
                .find_by_code(LANA_JOURNAL_CODE.to_string())
                .await?;
            Ok(JournalInit {
                journal_id: journal.id,
            })
        }
        Err(e) => Err(e.into()),
        Ok(journal) => Ok(JournalInit {
            journal_id: journal.id,
        }),
    }
}
