#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use tokio::runtime::Builder;

const THREAD_STACK_SIZE: usize = 4 * 1024 * 1024; // 4 MiB

fn main() -> anyhow::Result<()> {
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(THREAD_STACK_SIZE)
        .build()?;
    runtime.block_on(async { lana_cli::run().await })
}
