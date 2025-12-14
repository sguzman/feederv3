use std::sync::Arc;

use crate::domain::model::AppConfig;
use crate::ports::{clock::Clock, http::Http, random::RandomSource, repo::Repo};

pub struct AppContext<R, H, C, G>
where
    R: Repo,
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
