#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

fn main() -> anyhow::Result<()> {
    entity_rollups::run()
}
