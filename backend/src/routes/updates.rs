use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub status: String,
    pub message: String,
    pub progress: u8,
    pub version: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub changelog: Option<String>,
    pub release_url: String,
    pub is_prerelease: bool,
    pub latest_beta_version: Option<String>,
    pub beta_release_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRequest {
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateResponse {
    pub success: bool,
    pub message: String,
}

/// Get current version and check for updates
#[utoipa::path(
    get,
    path = "/api/updates/check",
    responses(
        (status = 200, description = "Version information retrieved successfully", body = VersionInfo),
        (status = 500, description = "Failed to check for updates")
    ),
    tag = "Updates"
)]
pub async fn check_updates(State(_state): State<AppState>) -> Result<Json<VersionInfo>, AppError> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    // Fetch latest release from GitHub
    let client = reqwest::Client::builder()
        .user_agent("CSF-Core-Updater")
        .build()
        .map_err(|e| AppError::InternalError(format!("Failed to create HTTP client: {}", e)))?;

    // Get latest stable release
    let response = client
        .get("https://api.github.com/repos/CS-Foundry/CSF-Core/releases/latest")
        .send()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to fetch releases: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::InternalError(format!(
            "GitHub API returned status: {}",
            response.status()
        )));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to parse release data: {}", e)))?;

    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let update_available = version_compare(&current_version, &latest_version);

    // Check for beta releases
    let all_releases_response = client
        .get("https://api.github.com/repos/CS-Foundry/CSF-Core/releases")
        .send()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to fetch all releases: {}", e)))?;

    let all_releases: Vec<GitHubRelease> = all_releases_response
        .json()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to parse releases data: {}", e)))?;

    // Find latest beta release
    let latest_beta = all_releases.iter().find(|r| r.prerelease).map(|r| {
        (
            r.tag_name.trim_start_matches('v').to_string(),
            r.html_url.clone(),
        )
    });

    Ok(Json(VersionInfo {
        current_version,
        latest_version: latest_version.clone(),
        update_available,
        changelog: Some(release.body),
        release_url: release.html_url,
        is_prerelease: release.prerelease,
        latest_beta_version: latest_beta.as_ref().map(|(v, _)| v.clone()),
        beta_release_url: latest_beta.map(|(_, url)| url),
    }))
}

/// Get update status
#[utoipa::path(
    get,
    path = "/api/updates/status",
    responses(
        (status = 200, description = "Update status retrieved successfully", body = UpdateStatus),
        (status = 404, description = "No update in progress"),
        (status = 500, description = "Failed to read status")
    ),
    tag = "Updates"
)]
pub async fn get_update_status(
    State(_state): State<AppState>,
) -> Result<Json<UpdateStatus>, AppError> {
    // Use /var/tmp instead of /tmp to avoid systemd PrivateTmp isolation
    let status_file = "/var/tmp/csf-core-update-status.json";

    match tokio::fs::read_to_string(status_file).await {
        Ok(content) => match serde_json::from_str::<UpdateStatus>(&content) {
            Ok(status) => Ok(Json(status)),
            Err(e) => Err(AppError::InternalError(format!(
                "Failed to parse status file: {}",
                e
            ))),
        },
        Err(_) => {
            // No status file means no update in progress
            Ok(Json(UpdateStatus {
                status: "idle".to_string(),
                message: "No update in progress".to_string(),
                progress: 0,
                version: None,
                timestamp: None,
            }))
        }
    }
}

/// Trigger update installation
#[utoipa::path(
    post,
    path = "/api/updates/install",
    request_body = UpdateRequest,
    responses(
        (status = 200, description = "Update initiated successfully", body = UpdateResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Failed to start update")
    ),
    tag = "Updates"
)]
pub async fn install_update(
    State(_state): State<AppState>,
    Json(payload): Json<UpdateRequest>,
) -> Result<Json<UpdateResponse>, AppError> {
    tracing::info!(
        "Update installation requested for version: {}",
        payload.version
    );

    let current_version = env!("CARGO_PKG_VERSION").to_string();
    tracing::info!("Current version: {}", current_version);

    // Safety check: don't downgrade
    if !version_compare(&current_version, &payload.version) {
        tracing::warn!(
            "Update rejected: Cannot install version {} (current: {})",
            payload.version,
            current_version
        );
        return Ok(Json(UpdateResponse {
            success: false,
            message: "Cannot install an older or same version".to_string(),
        }));
    }

    // Find the update script - try multiple locations
    let mut possible_paths: Vec<std::path::PathBuf> = vec![
        // Production path (daemon service)
        std::path::PathBuf::from("/opt/csf-core/scripts/update.sh"),
    ];

    // Development paths
    if let Ok(dir) = std::env::current_dir() {
        // When running from /opt/csf-core/backend
        possible_paths.push(dir.join("../scripts/update.sh"));
        // When running from project root
        possible_paths.push(dir.join("scripts/update.sh"));
        // When running from backend directory
        possible_paths.push(dir.join("../../scripts/update.sh"));
    }

    tracing::debug!("Searching for update script in: {:?}", possible_paths);

    let script_path = match possible_paths.iter().find(|&p| p.exists()) {
        Some(path) => path.clone(),
        None => {
            let error_msg = format!(
                "Update script not found in any expected location. Searched paths: {:?}. Note: Updates can only be performed in production installations, not during local development.",
                possible_paths
            );
            tracing::error!("{}", error_msg);
            return Err(AppError::InternalError(
                "Update functionality is only available in production installations. The update script was not found on this system.".to_string()
            ));
        }
    };

    tracing::info!("Found update script at: {:?}", script_path);

    // Clone version for use in response message
    let version_for_message = payload.version.clone();

    // Create initial status file to indicate update has started
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let status_content = format!(
        r#"{{
  "status": "in_progress",
  "message": "Initializing update process...",
  "progress": 0,
  "version": "{}",
  "timestamp": "{}"
}}"#,
        payload.version, timestamp
    );

    // Use /var/tmp instead of /tmp to avoid systemd PrivateTmp isolation
    if let Err(e) = tokio::fs::write("/var/tmp/csf-core-update-status.json", status_content).await {
        tracing::warn!("Failed to create initial status file: {}", e);
    }

    // Start update process in background with proper output handling
    let script_path_clone = script_path.clone();
    let version_clone = payload.version.clone();

    tokio::spawn(async move {
        tracing::info!("Spawning update process for version {}", version_clone);

        // Log current user for debugging
        let current_user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        tracing::info!("Running as user: {}", current_user);

        // Check if we're running as root
        let is_root = std::fs::metadata("/etc/shadow")
            .and_then(|_| std::fs::File::open("/etc/shadow"))
            .is_ok();

        tracing::info!("Running as root: {}", is_root);

        // Build command - use sudo nohup to detach from parent process
        // This ensures the update continues even when the backend service is stopped
        // sudoers entry: csf-core ALL=(ALL) NOPASSWD: /usr/bin/nohup /bin/bash /opt/csf-core/scripts/update.sh*
        let mut command = if is_root {
            // Running as root, execute script directly with nohup
            let mut cmd = Command::new("nohup");
            cmd.arg("/bin/bash");
            cmd.arg(&script_path_clone);
            cmd
        } else {
            // Running as csf-core user - use sudo nohup (not nohup sudo!)
            // This matches the sudoers entry exactly
            let mut cmd = Command::new("sudo");
            cmd.arg("/usr/bin/nohup");
            cmd.arg("/bin/bash");
            cmd.arg(&script_path_clone);
            cmd
        };

        // Set up environment variables that the script might need
        command.env(
            "PATH",
            "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        );
        command.env(
            "HOME",
            std::env::var("HOME").unwrap_or_else(|_| "/root".to_string()),
        );
        command.env("LANG", "C.UTF-8");
        command.env("LC_ALL", "C.UTF-8");

        // Set working directory to script location
        if let Some(script_dir) = script_path_clone.parent() {
            command.current_dir(script_dir);
            tracing::info!("Working directory: {:?}", script_dir);
        }

        command
            .arg(&version_clone)
            // Redirect stdout/stderr to log file for debugging
            .stdin(std::process::Stdio::null());

        // Keep stdout/stderr open so we can see errors in the log
        // The script itself redirects to /var/tmp/csf-core-update.log

        tracing::info!("Executing command: {:?}", command);
        tracing::info!(
            "Command environment: PATH={}, HOME={}",
            std::env::var("PATH").unwrap_or_default(),
            std::env::var("HOME").unwrap_or_default()
        );

        match command.spawn() {
            Ok(child) => {
                tracing::info!(
                    "Update process started successfully (PID: {:?}) and detached",
                    child.id()
                );

                // Process is detached - it will continue even if backend stops
                // Users can monitor progress via /var/tmp/csf-core-update-status.json
                tracing::info!(
                    "Update running in background. Monitor: /var/tmp/csf-core-update-status.json"
                );
            }
            Err(e) => {
                tracing::error!("Failed to start update process: {}", e);

                // Write error to status file
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let error_status = format!(
                    r#"{{
  "status": "error",
  "message": "Failed to start update script: {}",
  "progress": 0,
  "version": "{}",
  "timestamp": "{}"
}}"#,
                    e, version_clone, ts
                );

                // Use /var/tmp instead of /tmp
                let _ =
                    tokio::fs::write("/var/tmp/csf-core-update-status.json", error_status).await;
            }
        }
    });

    Ok(Json(UpdateResponse {
        success: true,
        message: format!(
            "Update to version {} initiated. The application will restart shortly.",
            version_for_message
        ),
    }))
}

/// Get changelog for a specific version
#[utoipa::path(
    get,
    path = "/api/updates/changelog/{version}",
    params(
        ("version" = String, Path, description = "Version to get changelog for")
    ),
    responses(
        (status = 200, description = "Changelog retrieved successfully"),
        (status = 404, description = "Version not found"),
        (status = 500, description = "Failed to fetch changelog")
    ),
    tag = "Updates"
)]
pub async fn get_changelog(
    State(_state): State<AppState>,
    axum::extract::Path(version): axum::extract::Path<String>,
) -> Result<Json<String>, AppError> {
    let client = reqwest::Client::builder()
        .user_agent("CSF-Core-Updater")
        .build()
        .map_err(|e| AppError::InternalError(format!("Failed to create HTTP client: {}", e)))?;

    let url = format!(
        "https://api.github.com/repos/CS-Foundry/CSF-Core/releases/tags/v{}",
        version.trim_start_matches('v')
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to fetch release: {}", e)))?;

    if response.status() == 404 {
        return Err(AppError::NotFound(format!("Version {} not found", version)));
    }

    if !response.status().is_success() {
        return Err(AppError::InternalError(format!(
            "GitHub API returned status: {}",
            response.status()
        )));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to parse release data: {}", e)))?;

    Ok(Json(release.body))
}

// Helper structs for GitHub API
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: String,
    html_url: String,
    prerelease: bool,
}

// Compare versions (returns true if v2 is newer than v1)
fn version_compare(v1: &str, v2: &str) -> bool {
    let parse_version =
        |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse::<u32>().ok()).collect() };

    let v1_parts = parse_version(v1);
    let v2_parts = parse_version(v2);

    for i in 0..v1_parts.len().max(v2_parts.len()) {
        let p1 = v1_parts.get(i).copied().unwrap_or(0);
        let p2 = v2_parts.get(i).copied().unwrap_or(0);

        if p2 > p1 {
            return true;
        } else if p2 < p1 {
            return false;
        }
    }

    false
}

// Error handling
#[derive(Debug)]
pub enum AppError {
    InternalError(String),
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
            AppError::NotFound(msg) => {
                tracing::warn!("Not found: {}", msg);
                (StatusCode::NOT_FOUND, msg.clone())
            }
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/updates/check", get(check_updates))
        .route("/updates/status", get(get_update_status))
        .route("/updates/install", post(install_update))
        .route("/updates/changelog/:version", get(get_changelog))
}
