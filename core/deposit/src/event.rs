use serde::{Deserialize, Serialize};

#[cfg(feature = "avro")]
use apache_avro::AvroSchema;

use super::primitives::{DepositAccountHolderId, DepositAccountId, DepositId, WithdrawalId};
use core_money::UsdCents;

#[cfg(feature = "avro")]
use apache_avro::schema::Schema as AvroSchemaType;

// Manual AvroSchema implementations for ID types (UUID wrappers)
#[cfg(feature = "avro")]
impl AvroSchema for DepositAccountId {
    fn get_schema() -> AvroSchemaType {
        AvroSchemaType::Uuid
    }
}

#[cfg(feature = "avro")]
impl AvroSchema for DepositAccountHolderId {
    fn get_schema() -> AvroSchemaType {
        AvroSchemaType::Uuid
    }
}

#[cfg(feature = "avro")]
impl AvroSchema for DepositId {
    fn get_schema() -> AvroSchemaType {
        AvroSchemaType::Uuid
    }
}

#[cfg(feature = "avro")]
impl AvroSchema for WithdrawalId {
    fn get_schema() -> AvroSchemaType {
        AvroSchemaType::Uuid
    }
}

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "avro", derive(avro_derive::AvroEventSchema))]
#[cfg_attr(feature = "avro", avro(namespace = "lana.core.deposit"))]
#[serde(tag = "type")]
pub enum CoreDepositEvent {
    DepositAccountCreated {
        id: DepositAccountId,
        account_holder_id: DepositAccountHolderId,
    },
    DepositInitialized {
        id: DepositId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
    },
    WithdrawalConfirmed {
        id: WithdrawalId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
    },
    DepositReverted {
        id: DepositId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
    },
    DepositAccountFrozen {
        id: DepositAccountId,
        account_holder_id: DepositAccountHolderId,
    },
}

#[cfg(all(test, feature = "avro"))]
mod avro_tests {
    use super::*;
    use apache_avro::Schema;

    #[test]
    fn test_schema_generation() {
        let schema = CoreDepositEvent::get_schema();

        // Verify schema is valid and has expected structure
        assert!(matches!(schema, Schema::Record(_)));

        let canonical = schema.canonical_form();
        assert!(canonical.contains("CoreDepositEvent"));
        assert!(canonical.contains("lana.core.deposit"));
        assert!(canonical.contains("CoreDepositEventType"));
    }

    #[test]
    fn test_schema_contains_all_event_types() {
        let schema = CoreDepositEvent::get_schema();
        let canonical = schema.canonical_form();

        // All event types should be in the schema
        assert!(canonical.contains("DepositAccountCreated"));
        assert!(canonical.contains("DepositInitialized"));
        assert!(canonical.contains("WithdrawalConfirmed"));
        assert!(canonical.contains("DepositReverted"));
        assert!(canonical.contains("DepositAccountFrozen"));
    }

    #[test]
    fn test_schema_contains_expected_fields() {
        let schema = CoreDepositEvent::get_schema();
        let canonical = schema.canonical_form();

        // Fields should be present
        assert!(canonical.contains("\"name\":\"type\""));
        assert!(canonical.contains("\"name\":\"id\""));
        assert!(canonical.contains("\"name\":\"amount\""));
        assert!(canonical.contains("\"name\":\"deposit_account_id\""));
        assert!(canonical.contains("\"name\":\"account_holder_id\""));
    }

    #[test]
    fn test_serialize_deposit_account_created_event() {
        // Create a test event
        let event = CoreDepositEvent::DepositAccountCreated {
            id: DepositAccountId::new(),
            account_holder_id: DepositAccountHolderId::new(),
        };

        // Serialize to JSON first to get the serde representation
        let json = serde_json::to_value(&event).expect("serialize to json");

        // Verify the JSON has the expected structure
        assert_eq!(json["type"], "DepositAccountCreated");
        assert!(json["id"].is_string());
        assert!(json["account_holder_id"].is_string());
    }

    #[test]
    fn test_serialize_deposit_initialized_event() {
        let event = CoreDepositEvent::DepositInitialized {
            id: DepositId::new(),
            deposit_account_id: DepositAccountId::new(),
            amount: UsdCents::from(10000u64),
        };

        let json = serde_json::to_value(&event).expect("serialize to json");

        assert_eq!(json["type"], "DepositInitialized");
        assert!(json["id"].is_string());
        assert!(json["deposit_account_id"].is_string());
        assert_eq!(json["amount"], 10000);
    }

    #[test]
    fn test_schema_fingerprint_stability() {
        // Schema fingerprints should be stable across invocations
        let schema1 = CoreDepositEvent::get_schema();
        let schema2 = CoreDepositEvent::get_schema();

        assert_eq!(schema1.canonical_form(), schema2.canonical_form());
    }
}
