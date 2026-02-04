mod helpers;

use es_entity::clock::{ArtificialClockConfig, ClockHandle};

use core_access::{CoreAccessEvent, PermissionSetId};
use helpers::event;

#[tokio::test]
async fn create_role_publishes_event() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let (clock, _time) = ClockHandle::artificial(ArtificialClockConfig::manual());
    let (access, subject, outbox) = helpers::init_access(&pool, clock).await?;

    let role_name = format!("test-role-{}", uuid::Uuid::new_v4());

    // Execute use case and wait for the expected event
    let (role, recorded) = event::expect_event(
        &outbox,
        || access.create_role(&subject, role_name.clone(), Vec::<PermissionSetId>::new()),
        |result, e| match e {
            CoreAccessEvent::RoleCreated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, role.id);
    assert_eq!(recorded.name, role_name);

    Ok(())
}
