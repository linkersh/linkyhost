use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use lazy_static::lazy_static;
use libvips::ops;
use libvips::VipsImage;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::BufReader;
use tokio::sync::OnceCell;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::state::AppState;

lazy_static! {
    pub static ref STORAGE_DIR: OnceCell<String> = OnceCell::new();
}

fn calculate_scale(src_width: u32, src_height: u32, dst_width: u32, dst_height: u32) -> f64 {
    let width_scale = dst_width as f64 / src_width as f64;
    let height_scale = dst_height as f64 / src_height as f64;

    // Return the minimum scale to preserve aspect ratio
    width_scale.min(height_scale)
}

pub struct ThumbOptions {
    pub width: u32,
    pub height: u32,
    pub vault_id: i64,
    pub file_id: i64,
}

pub struct Thumbnail {
    state: AppState,
    options: ThumbOptions,
}

impl Thumbnail {
    pub fn new(options: ThumbOptions, state: AppState) -> Thumbnail {
        Thumbnail { options, state }
    }

    pub async fn process(&self) -> anyhow::Result<PathBuf> {
        let thumb_path = self
            .state
            .store
            .get_thumb_path(self.options.vault_id, self.options.file_id);

        if !thumb_path.exists() {
            self.resize().await?;
        }

        Ok(thumb_path)
    }

    async fn resize(&self) -> anyhow::Result<PathBuf> {
        let state_cl = Arc::clone(&self.state);
        let path = state_cl
            .store
            .get_file_path(self.options.vault_id, self.options.file_id);

        let dst_height = self.options.height;
        let dst_width = self.options.width;

        let tmp_file_id = tokio::task::spawn_blocking(move || -> anyhow::Result<Uuid> {
            let image = VipsImage::new_from_file(path.to_str().unwrap())?;
            let src_height = image.get_height();
            let src_width = image.get_width();
            let scale = calculate_scale(src_width as u32, src_height as u32, dst_width, dst_height);

            let resized = ops::resize(&image, scale)?;
            let options = ops::JpegsaveOptions {
                q: 90,
                background: vec![255.0],
                optimize_coding: true,
                optimize_scans: true,
                interlace: true,
                ..ops::JpegsaveOptions::default()
            };

            let tmp_file_id = Uuid::new_v4();
            // we dont need to create a temp file lol
            // let tmp = state_cl.store.create_temp_file_sync(tmp_file_id)?;

            let temp_file_path = state_cl.store.get_temp_path(tmp_file_id);
            let temp_file_path = temp_file_path.to_str().unwrap();
            ops::jpegsave_with_opts(&resized, temp_file_path, &options)?;
            Ok(tmp_file_id)
        })
        .await??;

        let thumb_path = self
            .state
            .store
            .get_thumb_path(self.options.vault_id, self.options.file_id);
        let temp_file_path = self.state.store.get_temp_path(tmp_file_id);
        tokio::fs::rename(&temp_file_path, &thumb_path).await?;

        Ok(thumb_path)
    }
}

pub struct FsStore {
    base_dir: String,
}

impl FsStore {
    pub fn new(base_dir: &str, cancel: CancellationToken) -> Result<FsStore> {
        STORAGE_DIR.set(base_dir.to_owned())?;
        let cancel_copy = cancel.clone();
        let base_dir_copy = base_dir.to_owned();
        tokio::spawn(async move {
            tracing::info!("garbage collector started");
            let mut interval = tokio::time::interval(Duration::from_secs(5 * 60)); // every 5 minutes

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(error) = FsStore::collect_garbaj(&base_dir_copy).await {
                            tracing::error!(error = ?error, "failed to collect garbage files");
                        }
                    }
                    _ = cancel_copy.cancelled() => {
                        break;
                    }
                }
            }
        });

        Ok(Self {
            base_dir: base_dir.to_string(),
        })
    }

    // i love your content :)
    pub async fn collect_garbaj(base_dir: &str) -> anyhow::Result<()> {
        let mut entries = fs::read_dir(format!("{}/temp/", base_dir)).await?;
        loop {
            let entry = entries.next_entry().await?;
            let Some(en) = entry else {
                break;
            };

            let accessed = en.metadata().await?.accessed()?;
            let elapsed = accessed.elapsed()?;
            if elapsed > Duration::from_secs(60 * 5) {
                tracing::info!("deleted left over garbage: {:? }", en.path());
                if let Err(error) = fs::remove_file(en.path()).await {
                    tracing::error!(error = ?error, "failed to remove temp file")
                }
            }
        }
        Ok(())
    }

    // pub fn create_temp_file_sync(&self, id: Uuid) -> Result<std::fs::File> {
    //     let temp_dir = format!("{}/temp", self.base_dir);
    //     std::fs::create_dir_all(&temp_dir)?;

    //     let file = std::fs::OpenOptions::new()
    //         .create_new(true)
    //         .write(true)
    //         .read(true)
    //         .open(format!("{temp_dir}/{id}"))?;
    //     Ok(file)
    // }

    pub async fn create_temp_file(&self, id: Uuid) -> Result<File> {
        let temp_dir = format!("{}/temp", self.base_dir);
        fs::create_dir_all(&temp_dir).await?;

        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .open(format!("{temp_dir}/{id}"))
            .await?;
        Ok(file)
    }

    pub async fn upload_file_from_path(
        &self,
        vault_id: i64,
        file_id: i64,
        path: &Path,
    ) -> Result<()> {
        let vault_dir = format!("{}/vaults/{}/{}", self.base_dir, vault_id, file_id);
        fs::create_dir_all(&vault_dir).await?;

        let dest_path = self.get_file_path(vault_id, file_id);
        fs::rename(path, &dest_path).await?;

        Ok(())
    }

    pub async fn delete_file(&self, vault_id: i64, file_id: i64) -> Result<()> {
        let path = self.get_file_dir_path(vault_id, file_id);
        fs::remove_file(path).await?;
        Ok(())
    }

    pub async fn delete_vault(&self, vault_id: i64) -> Result<()> {
        let path = format!("{}/vaults/{}", self.base_dir, vault_id);
        fs::remove_dir_all(path).await?;
        Ok(())
    }

    pub fn get_file_dir_path(&self, vault_id: i64, file_id: i64) -> PathBuf {
        format!("{}/vaults/{}/{}", self.base_dir, vault_id, file_id).into()
    }

    pub fn get_file_path(&self, vault_id: i64, file_id: i64) -> PathBuf {
        format!(
            "{}/vaults/{}/{}/original.bin",
            self.base_dir, vault_id, file_id
        )
        .into()
    }

    pub fn get_thumb_path(&self, vault_id: i64, file_id: i64) -> PathBuf {
        format!(
            "{}/vaults/{}/{}/thumb_256x256.bin",
            self.base_dir, vault_id, file_id
        )
        .into()
    }

    pub fn get_temp_path(&self, file_id: Uuid) -> PathBuf {
        format!("{}/temp/{}", self.base_dir, file_id).into()
    }

    pub async fn get_file_reader(&self, vault_id: i64, file_id: i64) -> Result<BufReader<File>> {
        let path = self.get_file_path(vault_id, file_id);
        let file = fs::File::open(path).await?;
        Ok(BufReader::new(file))
    }
}
