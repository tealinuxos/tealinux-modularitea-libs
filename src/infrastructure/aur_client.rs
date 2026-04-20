//! AUR RPC API Client
//!
//! HTTP client for querying the Arch User Repository RPC API v5.
//! Includes TTL-based in-memory search cache to avoid redundant network requests.

use crate::domain::aur_package::{AurPackage, AurRpcResponse};
use crate::error::{ModulariteaError, Result};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const AUR_RPC_BASE: &str = "https://aur.archlinux.org/rpc/";
/// Cache TTL: 5 minutes
const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

// --- Global HTTP Client (connection pooling) ---
static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(reqwest::Client::new)
}

// --- In-memory search result cache ---
type SearchCache = Mutex<HashMap<String, (Vec<AurPackage>, Instant)>>;

static SEARCH_CACHE: OnceLock<SearchCache> = OnceLock::new();

fn search_cache() -> &'static SearchCache {
    SEARCH_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct AurClient;

impl AurClient {
    /// Search AUR packages by keyword.
    ///
    /// Results are cached in-memory for 5 minutes to reduce latency on repeated queries.
    /// Uses: `https://aur.archlinux.org/rpc/?v=5&type=search&arg=<query>`
    pub async fn search(query: &str) -> Result<Vec<AurPackage>> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        let cache_key = trimmed.to_lowercase();

        // --- Check cache ---
        {
            let cache = search_cache().lock().unwrap();
            if let Some((cached_results, cached_at)) = cache.get(&cache_key) {
                if cached_at.elapsed() < CACHE_TTL {
                    return Ok(cached_results.clone());
                }
            }
        }

        // --- Cache miss: fetch from AUR RPC ---
        let url = format!("{}?v=5&type=search&arg={}", AUR_RPC_BASE, trimmed);

        let response = client().get(&url).send().await.map_err(|e| {
            ModulariteaError::InternalError(format!("AUR API request failed: {}", e))
        })?;

        let rpc_response: AurRpcResponse = response
            .json()
            .await
            .map_err(|e| ModulariteaError::InternalError(format!("AUR API parse failed: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(ModulariteaError::InternalError(format!(
                "AUR API error: {}",
                error
            )));
        }

        let results = rpc_response.results;

        // --- Store in cache ---
        {
            let mut cache = search_cache().lock().unwrap();
            cache.insert(cache_key, (results.clone(), Instant::now()));
        }

        Ok(results)
    }

    /// Get detailed info for one or more packages by name
    ///
    /// Uses: `https://aur.archlinux.org/rpc/?v=5&type=info&arg[]=<pkg1>&arg[]=<pkg2>`
    pub async fn info(packages: &[&str]) -> Result<Vec<AurPackage>> {
        if packages.is_empty() {
            return Ok(Vec::new());
        }

        let mut url = format!("{}?v=5&type=info", AUR_RPC_BASE);
        for pkg in packages {
            url.push_str(&format!("&arg[]={}", pkg));
        }

        let response = client().get(&url).send().await.map_err(|e| {
            ModulariteaError::InternalError(format!("AUR API request failed: {}", e))
        })?;

        let rpc_response: AurRpcResponse = response
            .json()
            .await
            .map_err(|e| ModulariteaError::InternalError(format!("AUR API parse failed: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(ModulariteaError::InternalError(format!(
                "AUR API error: {}",
                error
            )));
        }

        Ok(rpc_response.results)
    }

    /// Get info for a single package
    pub async fn get_package(name: &str) -> Result<Option<AurPackage>> {
        let results = Self::info(std::slice::from_ref(&name)).await?;
        Ok(results.into_iter().next())
    }
}
