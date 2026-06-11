use std::sync::{Arc, Mutex};

use reqwest::Client;

/// Port the Python sidecar listens on.
static SIDECAR_PORT: u16 = 8321;
/// Maximum health check retries: 30 × 500ms = 15s budget.
static HEALTH_MAX_RETRIES: u32 = 30;
/// Delay between health check retries.
static HEALTH_RETRY_INTERVAL_MS: u64 = 500;

/// Manages the Python sidecar process and provides health-check capabilities.
pub struct AiBridge {
    sidecar_pid: Arc<Mutex<u32>>,
    http_client: Arc<Client>,
}

impl AiBridge {
    pub fn new() -> Self {
        Self {
            sidecar_pid: Arc::new(Mutex::new(0)),
            http_client: Arc::new(Client::new()),
        }
    }

    /// Resolve the ai-service directory by walking up from the current executable.
    fn resolve_sidecar_dir() -> Result<std::path::PathBuf, String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let mut dir = exe.parent().ok_or("No parent dir for executable")?;
        for _ in 0..6 {
            if dir.join("ai-service").is_dir() {
                return Ok(dir.join("ai-service"));
            }
            dir = dir.parent().ok_or("Could not find ai-service directory in any parent")?;
        }
        Err("ai-service directory not found in any parent of executable".into())
    }

    /// Spawn the Python sidecar process.
    pub fn spawn_sidecar(&self) -> Result<(), String> {
        let sidecar_dir = Self::resolve_sidecar_dir()?;
        log::info!("Spawning sidecar from: {}", sidecar_dir.display());

        let http_client = Arc::clone(&self.http_client);
        let sidecar_pid = Arc::clone(&self.sidecar_pid);

        // Spawn the sidecar in a blocking task so we don't block the Tauri main thread.
        tauri::async_runtime::spawn_blocking(move || {
            let mut child = match std::process::Command::new("python")
                .arg("-m")
                .arg("uvicorn")
                .arg("main:app")
                .current_dir(&sidecar_dir)
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to spawn sidecar: {}", e);
                    return;
                }
            };

            let pid = child.id();
            {
                let mut guard = sidecar_pid.lock().unwrap();
                *guard = pid;
            }
            log::info!("Sidecar spawned with PID {}", pid);

            // Wait for the child process to finish (runs until app exit).
            let _ = child.wait();
        });

        // Spawn the health-poll loop.
        let http_client2 = Arc::clone(&http_client);
        tauri::async_runtime::spawn(async move {
            let url = format!("http://127.0.0.1:{}/health", SIDECAR_PORT);
            for attempt in 1..=HEALTH_MAX_RETRIES {
                match http_client2.get(&url).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        log::info!("Sidecar health check succeeded on attempt {}", attempt);
                        return;
                    }
                    Ok(resp) => {
                        log::warn!(
                            "Sidecar health attempt {}/{} returned {}",
                            attempt,
                            HEALTH_MAX_RETRIES,
                            resp.status()
                        );
                    }
                    Err(e) => {
                        log::warn!(
                            "Sidecar health attempt {}/{} failed: {}",
                            attempt,
                            HEALTH_MAX_RETRIES,
                            e
                        );
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(HEALTH_RETRY_INTERVAL_MS)).await;
            }
            log::error!("Sidecar health check failed after {} retries", HEALTH_MAX_RETRIES);
        });

        Ok(())
    }

    /// Call the sidecar /health endpoint and return its JSON response.
    pub async fn sidecar_health(&self) -> Result<serde_json::Value, String> {
        let url = format!("http://127.0.0.1:{}/health", SIDECAR_PORT);

        let resp = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Sidecar request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Sidecar returned status {}", resp.status()));
        }

        let body = resp
            .json::<serde_json::Value>()
            .await
            .map_err(|e| format!("Failed to parse sidecar response: {}", e))?;

        Ok(body)
    }

    /// Kill the sidecar process on cleanup.
    pub fn stop_sidecar(&self) {
        let pid = {
            let guard = self.sidecar_pid.lock().unwrap();
            if *guard == 0 {
                return;
            }
            *guard
        };

        #[cfg(target_family = "unix")]
        {
            let _ = std::process::Command::new("kill")
                .arg(&pid.to_string())
                .output();
        }

        #[cfg(target_family = "windows")]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .output();
        }
    }
}

/// Tauri command: start the sidecar process (idempotent).
#[tauri::command]
pub fn start_sidecar(_app: tauri::AppHandle, state: tauri::State<'_, AiBridge>) -> Result<String, String> {
    state.spawn_sidecar()?;
    Ok("Sidecar started".into())
}

/// Tauri command: check sidecar health.
#[tauri::command]
pub async fn sidecar_health_command(
    state: tauri::State<'_, AiBridge>,
) -> Result<serde_json::Value, String> {
    state.sidecar_health().await
}