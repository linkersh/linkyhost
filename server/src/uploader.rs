use std::sync::Arc;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use tokio::{fs::File, io::BufWriter, sync::Mutex};
use uuid::Uuid;

pub struct Upload {
    pub id: Uuid,
    pub user_id: i32,
    pub expected_size: usize,
    pub received_size: usize,
    pub last_received: DateTime<Utc>,
    pub content_type: String,
    pub file_name: String,
    pub tmp_file: BufWriter<File>,
    pub vault_id: i64,
}

pub struct CreateUploadInfo {
    pub upload_id: Uuid,
    pub vault_id: i64,
    pub user_id: i32,
    pub file_size: usize,
    pub file_name: String,
    pub file_type: String,
    pub tmp_file: File,
}

pub struct Uploader {
    uploads: DashMap<Uuid, Arc<Mutex<Upload>>>,
}

impl Uploader {
    pub fn new() -> Uploader {
        Uploader {
            uploads: DashMap::new(),
        }
    }

    pub async fn create_upload(&self, info: CreateUploadInfo) -> anyhow::Result<()> {
        let tmp_file = BufWriter::new(info.tmp_file);
        let upload = Upload {
            id: info.upload_id,
            expected_size: info.file_size,
            received_size: 0,
            last_received: Utc::now(),
            content_type: info.file_type,
            file_name: info.file_name,
            tmp_file,
            vault_id: info.vault_id,
            user_id: info.user_id,
        };
        self.uploads.insert(info.upload_id, Arc::new(Mutex::new(upload)));

        tracing::info!(
            "starting upload to vault id: {}, upload id: {}",
            info.vault_id,
            info.upload_id
        );
        Ok(())
    }

    pub async fn get_upload(&self, upload_id: Uuid) -> Option<Arc<Mutex<Upload>>> {
        let upload = self.uploads.get(&upload_id).map(|x| Arc::clone(&x));
        upload
    }
}
