use std::sync::Arc;
use std::time::Instant;

use futures::{stream, StreamExt};
use tracing::{debug, info, warn};

use crate::app::context::AppContext;
use crate::domain::link_state::LinkState;
use crate::domain::model::FeedConfig;
use crate::infra::time::format_epoch_ms;
use crate::ports::{clock::Clock, http::Http, random::RandomSource, repo::Repo};

use super::actions::{do_get, do_head};
use super::concurrency::ConcurrencyGuards;
use super::state::{describe_action, should_record_history, to_link_state};

pub async fn run_tick<R, H, C, G>(
    ctx: &AppContext<R, H, C, G>,
    concurrency: &ConcurrencyGuards,
    tick_started: Instant,
) -> Result<(), String>
where
    R: Repo + 'static,
    H: Http + 'static,
    C: Clock + 'static,
    G: RandomSource + 'static,
{
    let due_batch_size: i64 = 1000;
    let default_parallelism: usize = 64;

    let cfg = ctx.cfg.clone();

    let now_ms = ctx.clock.now_epoch_ms().await;
    let due_started = Instant::now();
    let due = ctx.repo.due_feeds(now_ms, due_batch_size).await?;
    let due_elapsed = due_started.elapsed();

    info!(
      tick_time = %format_epoch_ms(now_ms, &cfg.timezone),
      due = due.len(),
      due_batch_limit = due_batch_size,
      due_query_ms = due_elapsed.as_millis(),
      "Scheduler tick"
    );

    let parallelism = cfg
        .global_max_concurrent_requests
        .unwrap_or(default_parallelism);
    let repo = ctx.repo.clone();
    let http = ctx.http.clone();
    let clock = ctx.clock.clone();
    let rng = ctx.rng.clone();

    stream::iter(due)
        .map(|feed| {
            let cfg = cfg.clone();
            let repo = repo.clone();
            let http = http.clone();
            let clock = clock.clone();
            let rng = rng.clone();
            let concurrency = concurrency.clone();

            async move {
                if let Err(e) = process_feed(cfg, repo, http, clock, rng, concurrency, feed).await {
                    warn!(error = %e, "process_feed failed");
                }
            }
        })
        .buffer_unordered(parallelism)
        .collect::<Vec<_>>()
        .await;

    info!(
      tick_time = %format_epoch_ms(now_ms, &cfg.timezone),
      total_ms = tick_started.elapsed().as_millis(),
      "Scheduler tick complete"
    );

    Ok(())
}

async fn process_feed<R, H, C, G>(
    cfg: Arc<crate::domain::model::AppConfig>,
    repo: Arc<R>,
    http: Arc<H>,
    clock: Arc<C>,
    rng: Arc<G>,
    concurrency: ConcurrencyGuards,
    feed: FeedConfig,
) -> Result<(), String>
where
    R: Repo,
    H: Http,
    C: Clock,
    G: RandomSource,
{
    let now_ms = clock.now_epoch_ms().await;
    let rand = rng.next_f64().await;

    let stored = repo.latest_state(&feed.id).await?;
    let state = stored
        .and_then(|r| to_link_state(&r, &cfg))
        .unwrap_or_else(|| {
            LinkState::initial(
                feed.id.clone(),
                feed.base_poll_seconds,
                cfg.max_poll_seconds,
                cfg.jitter_fraction,
                now_ms,
            )
        });

    let action = LinkState::decide_next_action(&state, now_ms);

    debug!(
      feed_id = %feed.id,
      action = %describe_action(&action, &cfg),
      now = %format_epoch_ms(now_ms, &cfg.timezone),
      "Decided next action"
    );

    match action {
        crate::domain::link_state::NextAction::SleepUntil { .. } => Ok(()),
        crate::domain::link_state::NextAction::DoHead { state } => {
            let record_history = should_record_history(&cfg, rng.as_ref()).await;
            do_head(
                &cfg,
                &repo,
                &http,
                &concurrency,
                &feed,
                state,
                now_ms,
                rand,
                record_history,
            )
            .await
        }
        crate::domain::link_state::NextAction::DoGet { state } => {
            let record_history = should_record_history(&cfg, rng.as_ref()).await;
            do_get(
                &cfg,
                &repo,
                &http,
                &concurrency,
                &feed,
                state,
                now_ms,
                rand,
                record_history,
            )
            .await
        }
    }
}
