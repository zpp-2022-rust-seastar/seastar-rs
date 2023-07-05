use seastar::{submit_to, AppTemplate, Logger, Options};

#[ctor::ctor]
static MLOGGER: Logger = Logger::new("main");

#[ctor::ctor]
static CSLOGGER: Logger = Logger::new("cross-shard");

pub fn main() {
    let opts = Options::default();
    let shard_count = opts.get_smp();
    let mut template = AppTemplate::new_from_options(opts);
    template.run_void(std::env::args(), async move {
        seastar::info!(MLOGGER, "Starting application!");

        let futs = (0..shard_count)
            .map(|shard_id| {
                submit_to(shard_id, move || async move {
                    seastar::info!(CSLOGGER, "Hello from shard {}!", shard_id);
                })
            })
            .collect::<Vec<_>>();

        for fut in futs {
            fut.await;
        }

        seastar::info!(MLOGGER, "Stopping application!");
        Ok(())
    });
}
