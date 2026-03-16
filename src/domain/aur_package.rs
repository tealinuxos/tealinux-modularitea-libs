//! AUR Package domain model
//!
//! Represents an AUR package from the AUR RPC API.

use serde::{Deserialize, Serialize};

/// An AUR package as returned by the AUR RPC API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurPackage {
    /// Package name
    #[serde(rename = "Name")]
    pub name: String,

    /// Package version
    #[serde(rename = "Version")]
    pub version: String,

    /// Package description
    #[serde(rename = "Description")]
    pub description: Option<String>,

    /// Current maintainer
    #[serde(rename = "Maintainer")]
    pub maintainer: Option<String>,

    /// Number of votes
    #[serde(rename = "NumVotes")]
    pub num_votes: u32,

    /// Popularity score
    #[serde(rename = "Popularity")]
    pub popularity: f64,

    /// Timestamp when flagged out-of-date (None if not flagged)
    #[serde(rename = "OutOfDate")]
    pub out_of_date: Option<u64>,

    /// Upstream project URL
    #[serde(rename = "URL")]
    pub url: Option<String>,

    /// AUR package page URL base
    #[serde(rename = "PackageBase")]
    pub package_base: Option<String>,

    /// First submitted timestamp
    #[serde(rename = "FirstSubmitted")]
    pub first_submitted: Option<u64>,

    /// Last modified timestamp
    #[serde(rename = "LastModified")]
    pub last_modified: Option<u64>,

    /// Package license
    #[serde(rename = "License")]
    pub license: Option<Vec<String>>,

    /// Dependencies
    #[serde(rename = "Depends", default)]
    pub depends: Vec<String>,

    /// Make dependencies
    #[serde(rename = "MakeDepends", default)]
    pub make_depends: Vec<String>,

    /// Optional dependencies
    #[serde(rename = "OptDepends", default)]
    pub opt_depends: Vec<String>,

    /// Conflicts
    #[serde(rename = "Conflicts", default)]
    pub conflicts: Vec<String>,

    /// Provides
    #[serde(rename = "Provides", default)]
    pub provides: Vec<String>,
}

/// Response wrapper from the AUR RPC API
#[derive(Debug, Clone, Deserialize)]
pub struct AurRpcResponse {
    /// Number of results
    pub resultcount: u32,

    /// Result type
    #[serde(rename = "type")]
    pub response_type: String,

    /// Package results
    pub results: Vec<AurPackage>,

    /// Error message (if any)
    pub error: Option<String>,
}

/// A locally installed AUR (foreign) package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledAurPackage {
    /// Package name
    pub name: String,
    /// Installed version
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aur_package_deserialize() {
        let json = r#"{
            "Name": "paru",
            "Version": "2.0.4-1",
            "Description": "Feature packed AUR helper",
            "Maintainer": "Morganamilo",
            "NumVotes": 1234,
            "Popularity": 42.5,
            "OutOfDate": null,
            "URL": "https://github.com/Morganamilo/paru",
            "PackageBase": "paru",
            "FirstSubmitted": 1600000000,
            "LastModified": 1700000000,
            "License": ["GPL3"],
            "Depends": ["pacman", "git"],
            "MakeDepends": ["cargo"],
            "OptDepends": [],
            "Conflicts": [],
            "Provides": []
        }"#;

        let pkg: AurPackage = serde_json::from_str(json).unwrap();
        assert_eq!(pkg.name, "paru");
        assert_eq!(pkg.num_votes, 1234);
        assert!(pkg.out_of_date.is_none());
    }

    #[test]
    fn test_aur_rpc_response_deserialize() {
        let json = r#"{
            "resultcount": 1,
            "type": "search",
            "results": [{
                "Name": "paru",
                "Version": "2.0.4-1",
                "Description": "Feature packed AUR helper",
                "Maintainer": "Morganamilo",
                "NumVotes": 1234,
                "Popularity": 42.5,
                "OutOfDate": null,
                "URL": null,
                "PackageBase": "paru",
                "FirstSubmitted": 1600000000,
                "LastModified": 1700000000,
                "Depends": [],
                "MakeDepends": [],
                "OptDepends": [],
                "Conflicts": [],
                "Provides": []
            }]
        }"#;

        let response: AurRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.resultcount, 1);
        assert_eq!(response.results[0].name, "paru");
    }
}
