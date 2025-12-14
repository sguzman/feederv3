use std::{path::PathBuf, sync::Arc};

use feedrv3::app::{context::AppContext, scheduler::Scheduler};
use feedrv3::domain::model::{AppConfig, AppMode, FeedConfig};
use feedrv3::infra::{
    config::{ConfigLoader, LoadedConfig},
    logging::{init_logging, BootError},
    random::MutexRng,
    reqwest_http::ReqwestHttp,
    sqlite_repo::SqliteRepo,
    system_clock::SystemClock,
};
use feedrv3::ports::repo::Repo;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), BootError> {
    let args = parse_args();
    let cfg_path = pick_config_path(args.config_path);
    let LoadedConfig {
        app: app_cfg,
        feeds,
    } = ConfigLoader::load(&cfg_path)
        .await
        .map_err(|e| BootError::Fatal(e.to_string()))?;
    init_logging(&app_cfg.log_level);

    info!(timezone = %app_cfg.timezone, "Using timezone");
    info!(
      feeds = feeds.len(),
      db_path = %app_cfg.db_path.display(),
      mode = ?app_cfg.mode,
      "Loaded config"
    );

    if matches!(app_cfg.mode, AppMode::Dev) {
        warn!(db_path = %app_cfg.db_path.display(), "Dev mode enabled, deleting database");
        let _ = tokio::fs::remove_file(&app_cfg.db_path).await;
    }

    let repo = Arc::new(
        SqliteRepo::new(&app_cfg.db_path)
            .await
            .map_err(BootError::Fatal)?,
    );
    repo.migrate(&app_cfg.timezone, app_cfg.default_poll_seconds)
        .await
        .map_err(BootError::Fatal)?;

    let cfg = Arc::new(app_cfg);

    match args.mode {
        RunMode::IngestBenchmark { feeds_to_insert } => {
            if feeds_to_insert == 0 {
                return Err(BootError::Fatal(
                    "ingest benchmark requires a feed count > 0".into(),
                ));
            }
            info!(feeds = feeds_to_insert, "Starting ingest benchmark only");
            ingest_feeds(
                repo.clone(),
                cfg.clone(),
                benchmark_feed_stream(feeds_to_insert, cfg.default_poll_seconds),
            )
            .await?;
            info!(feeds = feeds_to_insert, "Ingest benchmark finished");
            return Ok(());
        }
        RunMode::Scheduler => {}
    }

    ingest_feeds(repo.clone(), cfg.clone(), feeds).await?;

    let http = Arc::new(
        ReqwestHttp::new(cfg.user_agent.clone()).map_err(|e| BootError::Fatal(e.to_string()))?,
    );
    let clock = Arc::new(SystemClock::default());
    let rng = Arc::new(MutexRng::new());

    let ctx = AppContext {
        cfg: cfg.clone(),
        repo: repo.clone(),
        http: http.clone(),
        clock: clock.clone(),
        rng: rng.clone(),
    };

    if let Err(e) = Scheduler::run_forever(ctx).await {
        error!(error = %e, "Fatal error");
        return Err(BootError::Fatal(e.to_string()));
    }

    Ok(())
}

fn pick_config_path(arg1: Option<String>) -> PathBuf {
    if let Some(p) = arg1 {
        return PathBuf::from(p);
    }

    // Prefer repo-local res/ config; fall back to old resources path for compatibility.
    let candidates = [
        PathBuf::from("res/config.toml"),
        PathBuf::from("src/main/resources/config/config.toml"),
    ];
    for p in &candidates {
        if p.exists() {
            return p.clone();
        }
    }
    candidates[0].clone()
}

enum RunMode {
    Scheduler,
    IngestBenchmark { feeds_to_insert: usize },
}

struct Args {
    config_path: Option<String>,
    mode: RunMode,
}

fn parse_args() -> Args {
    let mut args = std::env::args().skip(1);
    let mut config_path = None;
    let mut mode = RunMode::Scheduler;

    while let Some(arg) = args.next() {
        if arg == "--ingest-benchmark" {
            if let Some(n) = args.next() {
                let feeds_to_insert = n.parse::<usize>().unwrap_or(0);
                mode = RunMode::IngestBenchmark { feeds_to_insert };
            }
        } else {
            config_path = Some(arg);
        }
    }

    Args { config_path, mode }
}

async fn ingest_feeds<R, I>(repo: Arc<R>, cfg: Arc<AppConfig>, feeds: I) -> Result<(), BootError>
where
    R: Repo + 'static,
    I: IntoIterator<Item = FeedConfig> + Send,
    I::IntoIter: Send,
{
    // Large chunks keep transaction overhead low without blowing memory.
    let chunk_size = 10_000;
    repo.upsert_feeds_bulk(feeds, chunk_size, &cfg.timezone)
        .await
        .map_err(BootError::Fatal)
}

fn benchmark_feed_stream(
    count: usize,
    default_poll_seconds: u64,
) -> impl Iterator<Item = FeedConfig> {
    (0..count).map(move |i| FeedConfig {
        id: format!("bench-{i}"),
        url: format!("https://bench.example.com/{i}.xml"),
        domain: "bench.example.com".to_string(),
        base_poll_seconds: default_poll_seconds,
    })
}
