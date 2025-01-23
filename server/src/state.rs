use std::sync::Arc;

use anyhow::{Error, Result};
use libvips::VipsApp;

use crate::{auth::Auther, config::AppConfig, db::Database, store::FsStore, uploader::Uploader};

pub type AppState = Arc<AppStateRef>;

pub struct AppStateRef {
    pub database: Database,
    pub store: FsStore,
    pub config: AppConfig,
    pub auther: Auther,
    pub vips: VipsApp,
    pub uploader: Uploader,
}

pub async fn create_state(config: AppConfig) -> Result<AppState> {
    let store = if let Some(fs) = &config.store.fs {
        FsStore::new(&fs.base_dir)?
    } else {
        return Err(Error::msg("no storage service configured"));
    };

    let database = Database::new(&config).await?;
    let auther = Auther::new(&config.auth.secret)?;
    let uploader = Uploader::new();
    let vips = VipsApp::new("linkyhost", true)?;
    vips.concurrency_set(std::thread::available_parallelism()?.get() as i32);

    Ok(Arc::new(AppStateRef {
        store,
        database,
        config,
        auther,
        uploader,
        vips,
    }))
}
