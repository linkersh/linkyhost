use std::{
    io::BufWriter,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use fast_image_resize::{images::Image, IntoImageView, ResizeOptions, Resizer};
use image::{codecs::avif::AvifEncoder, ImageEncoder, ImageFormat, ImageReader};
use lazy_static::lazy_static;
use tokio::{
    fs::{self, File, OpenOptions},
    io::BufReader,
    sync::OnceCell,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::state::AppState;

lazy_static! {
    pub static ref STORAGE_DIR: OnceCell<String> = OnceCell::new();
}

pub struct ThumbOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub vault_id: i64,
    pub file_id: i64,
    pub content_type: String,
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
        let mime_type = self.options.content_type.clone();

        let tmp_file_id = tokio::task::spawn_blocking(move || -> anyhow::Result<Uuid> {
            let mut src_image = ImageReader::open(path.to_str().unwrap())?;
            src_image
                .set_format(ImageFormat::from_mime_type(&mime_type).unwrap_or(ImageFormat::Jpeg));

            let src_image = src_image.decode()?;
            let dst_width = dst_width.unwrap_or(src_image.width());
            let dst_height = dst_height.unwrap_or(src_image.height());

            let tmp_file_id = Uuid::new_v4();
            let temp_file_path = state_cl.store.get_temp_path(tmp_file_id);
            let temp_file_path = temp_file_path.to_str().unwrap();

            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(temp_file_path)?;
            let result_buf = BufWriter::new(file);
            let mut dst_image = Image::new(dst_width, dst_height, src_image.pixel_type().unwrap());
            let mut resizer = Resizer::new();
            resizer.resize(
                &src_image,
                &mut dst_image,
                &Some(ResizeOptions::new().fit_into_destination(None)),
            )?;

            let buffer = dst_image.into_vec();
            AvifEncoder::new_with_speed_quality(result_buf, 10, 60).write_image(
                &buffer,
                dst_width,
                dst_height,
                src_image.color().into(),
            )?;

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
