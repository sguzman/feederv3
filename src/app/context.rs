use std::sync::Arc;

use crate::domain::model::AppConfig;
use crate::ports::{clock::Clock, http::Http, random::RandomSource, repo::Repo};

/// Bundles the runtime dependencies the scheduler needs (configuration,
/// persistence, HTTP client, clock, and randomness source).
#[derive(Clone)]
pub struct AppContext<R, H, C, G>
where
    R: Repo + ?Sized,
    H: Http,
    C: Clock,
    G: RandomSource,
{
    pub cfg: Arc<AppConfig>,
    pub repo: Arc<R>,
    pub http: Arc<H>,
    pub clock: Arc<C>,
    pub rng: Arc<G>,
}
