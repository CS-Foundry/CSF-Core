use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub current_version: String,
    pub current_commit: Option<String>,
    pub latest_version: String,
    pub latest_commit: Option<String>,
    pub update_available: bool,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub published_at: String,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/system/version", get(check_version))
}

/// GET /api/system/version - Get current and latest version info
pub async fn check_version(
    State(_state): State<AppState>,
) -> Result<Json<VersionInfo>, StatusCode> {
    // Lese aktuelle Version aus VERSION-Datei
    let current_version = std::fs::read_to_string("/opt/csf-core/VERSION")
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());

    let current_commit = std::fs::read_to_string("/opt/csf-core/COMMIT")
        .ok()
        .map(|s| s.trim().to_string());

    // Prüfe neueste Version auf GitHub
    let client = match reqwest::Client::builder()
        .user_agent("CSF-Core-Update-Checker")
        .build()
    {
        Ok(c) => c,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let github_api_url = "https://api.github.com/repos/CS-Foundry/CSF-Core/releases/latest";

    match client.get(github_api_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<GitHubRelease>().await {
                    Ok(release) => {
                        let latest_version = release.tag_name.trim_start_matches('v').to_string();
                        let current_ver = current_version.trim_start_matches('v');

                        // Vergleiche Versionen (einfache String-Vergleich, könnte verbessert werden)
                        let update_available = latest_version != current_ver
                            && version_compare(&latest_version, current_ver) > 0;

                        // Finde passende Architektur
                        let arch = detect_architecture();
                        let download_url = release
                            .assets
                            .iter()
                            .find(|asset| asset.name.contains(&arch))
                            .map(|asset| asset.browser_download_url.clone());

                        Ok(Json(VersionInfo {
                            current_version: current_version.clone(),
                            current_commit,
                            latest_version: latest_version.clone(),
                            latest_commit: None,
                            update_available,
                            download_url,
                            release_notes: Some(release.body),
                            published_at: Some(release.published_at),
                        }))
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse GitHub release: {}", e);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            } else {
                tracing::warn!("GitHub API returned status: {}", response.status());
                // Keine neue Version verfügbar oder API-Fehler
                Ok(Json(VersionInfo {
                    current_version: current_version.clone(),
                    current_commit,
                    latest_version: current_version.clone(),
                    latest_commit: None,
                    update_available: false,
                    download_url: None,
                    release_notes: None,
                    published_at: None,
                }))
            }
        }
        Err(e) => {
            tracing::error!("Failed to check for updates: {}", e);
            // Bei Netzwerkfehler gib trotzdem aktuelle Version zurück
            Ok(Json(VersionInfo {
                current_version: current_version.clone(),
                current_commit,
                latest_version: current_version.clone(),
                latest_commit: None,
                update_available: false,
                download_url: None,
                release_notes: None,
                published_at: None,
            }))
        }
    }
}

fn detect_architecture() -> String {
    let arch = std::env::consts::ARCH;
    match arch {
        "x86_64" => "amd64".to_string(),
        "aarch64" => "arm64".to_string(),
        _ => "amd64".to_string(),
    }
}

fn version_compare(v1: &str, v2: &str) -> i8 {
    // Einfacher Versionsvergleich (v1 > v2 = 1, v1 == v2 = 0, v1 < v2 = -1)
    let v1_parts: Vec<u32> = v1.split('.').filter_map(|s| s.parse().ok()).collect();
    let v2_parts: Vec<u32> = v2.split('.').filter_map(|s| s.parse().ok()).collect();

    for i in 0..v1_parts.len().max(v2_parts.len()) {
        let p1 = v1_parts.get(i).copied().unwrap_or(0);
        let p2 = v2_parts.get(i).copied().unwrap_or(0);

        if p1 > p2 {
            return 1;
        } else if p1 < p2 {
            return -1;
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compare() {
        assert_eq!(version_compare("1.0.0", "1.0.0"), 0);
        assert_eq!(version_compare("1.0.1", "1.0.0"), 1);
        assert_eq!(version_compare("1.0.0", "1.0.1"), -1);
        assert_eq!(version_compare("2.0.0", "1.9.9"), 1);
        assert_eq!(version_compare("1.9.9", "2.0.0"), -1);
    }
}
