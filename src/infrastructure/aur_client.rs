//! AUR RPC API Client
//!
//! HTTP client for querying the Arch User Repository RPC API v5.

use crate::domain::aur_package::{AurPackage, AurRpcResponse};
use crate::error::{ModulariteaError, Result};

const AUR_RPC_BASE: &str = "https://aur.archlinux.org/rpc/";

pub struct AurClient;

impl AurClient {
    /// Search AUR packages by keyword
    ///
    /// Uses: `https://aur.archlinux.org/rpc/?v=5&type=search&arg=<query>`
    pub async fn search(query: &str) -> Result<Vec<AurPackage>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let url = format!("{}?v=5&type=search&arg={}", AUR_RPC_BASE, query);

        let response = reqwest::get(&url).await.map_err(|e| {
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

        let response = reqwest::get(&url).await.map_err(|e| {
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
        let results = Self::info(&[name]).await?;
        Ok(results.into_iter().next())
    }
}
