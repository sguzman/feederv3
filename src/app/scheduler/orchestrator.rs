use std::time::Instant;

use crate::app::context::AppContext;
use crate::ports::{clock::Clock, http::Http, random::RandomSource, repo::Repo};

use super::concurrency::ConcurrencyGuards;
use super::processing::run_tick;

pub struct Scheduler;

impl Scheduler {
    pub async fn run_forever<R, H, C, G>(ctx: AppContext<R, H, C, G>) -> Result<(), String>
    where
        R: Repo + 'static,
        H: Http + 'static,
        C: Clock + 'static,
        G: RandomSource + 'static,
    {
        let tick_interval = std::time::Duration::from_secs(5);
        let cfg = ctx.cfg.clone();
        let concurrency = ConcurrencyGuards::new(cfg.clone());
        let mut interval = tokio::time::interval(tick_interval);

        loop {
            interval.tick().await;
            let tick_started = Instant::now();
            run_tick(&ctx, &concurrency, tick_started).await?;
        }
    }
}
