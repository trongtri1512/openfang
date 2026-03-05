//! Portal database storage — independent PostgreSQL + JSON file fallback.
//!
//! Uses `PORTAL_DATABASE_URL` (separate from OpenFang's `DATABASE_URL`).

use crate::models::{PortalData, seed_defaults};
use std::path::PathBuf;


/// Portal application state — independent from OpenFang's AppState.
#[derive(Clone)]
pub struct PortalState {
    /// Portal data directory (e.g., ~/.openfang/portal/)
    pub data_dir: PathBuf,
    /// PostgreSQL connection pool (optional)
    pub db_pool: Option<deadpool_postgres::Pool>,
    /// OpenFang API base URL for system proxy calls
    pub openfang_api_url: String,
    /// OpenFang API key for authentication
    pub openfang_api_key: String,
}

impl PortalState {
    pub fn new() -> Self {
        let home = std::env::var("OPENFANG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs_fallback().join(".openfang")
            });
        let data_dir = home.join("portal");
        let _ = std::fs::create_dir_all(&data_dir);

        let db_pool = Self::init_pool();
        let openfang_api_url = std::env::var("OPENFANG_API_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:3000".into());
        let openfang_api_key = std::env::var("OPENFANG_API_KEY")
            .unwrap_or_default();

        Self { data_dir, db_pool, openfang_api_url, openfang_api_key }
    }

    fn init_pool() -> Option<deadpool_postgres::Pool> {
        let url = std::env::var("PORTAL_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .ok()?;
        let config: tokio_postgres::Config = url.parse().ok()?;
        let mgr_config = deadpool_postgres::ManagerConfig {
            recycling_method: deadpool_postgres::RecyclingMethod::Fast,
        };
        let mgr = deadpool_postgres::Manager::from_config(config, tokio_postgres::NoTls, mgr_config);
        let pool = deadpool_postgres::Pool::builder(mgr)
            .max_size(4)
            .build()
            .ok()?;

        // Ensure portal table exists
        let pool2 = pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Ok(client) = pool2.get().await {
                    let _ = client.execute(
                        "CREATE TABLE IF NOT EXISTS portal_data (id INTEGER PRIMARY KEY DEFAULT 1, data JSONB NOT NULL DEFAULT '{}'::jsonb, updated_at TIMESTAMPTZ DEFAULT NOW())",
                        &[],
                    ).await;
                }
            });
        });

        Some(pool)
    }

    fn data_path(&self) -> PathBuf {
        self.data_dir.join("portal_data.json")
    }
}

/// Fallback home directory
fn dirs_fallback() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Load portal data from PostgreSQL, fallback to JSON file.
pub fn load_data(state: &PortalState) -> PortalData {
    if let Some(ref pool) = state.db_pool {
        let pool = pool.clone();
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let client = pool.get().await.ok()?;
                let row = client
                    .query_opt("SELECT data FROM portal_data WHERE id = 1", &[])
                    .await
                    .ok()?;
                if let Some(row) = row {
                    let data: serde_json::Value = row.get(0);
                    serde_json::from_value(data).ok()
                } else {
                    Some(PortalData::default())
                }
            })
        });
        if let Some(data) = result {
            return data;
        }
        tracing::warn!("Failed to load portal data from PostgreSQL, falling back to JSON file");
    }
    load_from_file(state)
}

/// Save portal data to PostgreSQL (and JSON file as backup).
pub fn save_data(state: &PortalState, data: &PortalData) -> Result<(), String> {
    // Always save to JSON file as backup
    let _ = save_to_file(state, data);

    if let Some(ref pool) = state.db_pool {
        let pool = pool.clone();
        let json_val = serde_json::to_value(data).map_err(|e| e.to_string())?;
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let client = pool.get().await.map_err(|e| format!("DB pool: {e}"))?;
                client
                    .execute(
                        "INSERT INTO portal_data (id, data, updated_at) VALUES (1, $1, NOW()) ON CONFLICT (id) DO UPDATE SET data = $1, updated_at = NOW()",
                        &[&json_val],
                    )
                    .await
                    .map_err(|e| format!("DB save: {e}"))?;
                Ok::<(), String>(())
            })
        });
        if let Err(ref e) = result {
            tracing::warn!("PostgreSQL save failed: {e} (JSON backup was saved)");
        }
        return result;
    }
    Ok(())
}

fn load_from_file(state: &PortalState) -> PortalData {
    let path = state.data_path();
    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => PortalData::default(),
        }
    } else {
        PortalData::default()
    }
}

fn save_to_file(state: &PortalState, data: &PortalData) -> Result<(), String> {
    let path = state.data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

/// Initialize portal data with defaults if needed.
pub fn init_data(state: &PortalState) -> PortalData {
    let mut data = load_data(state);
    if seed_defaults(&mut data) {
        let _ = save_data(state, &data);
    }
    data
}
