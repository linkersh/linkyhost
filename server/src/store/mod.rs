use anyhow::Result;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::BufReader;
use uuid::Uuid;

pub struct FsStore {
    base_dir: String,
}

impl FsStore {
    pub fn new(base_dir: &str) -> Result<FsStore> {
        Ok(Self {
            base_dir: base_dir.to_string(),
        })
    }

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

    pub async fn upload_file(&self, vault_id: i64, file_id: i64, upload_id: Uuid) -> Result<()> {
        let vault_dir = format!("{}/vaults/{}", self.base_dir, vault_id);
        fs::create_dir_all(&vault_dir).await?;

        let dest_path = format!("{}/vaults/{}/{}", self.base_dir, vault_id, file_id);
        fs::rename(&format!("{}/temp/{upload_id}", self.base_dir), &dest_path).await?;

        Ok(())
    }

    pub async fn delete_file(&self, vault_id: i64, file_id: i64) -> Result<()> {
        let path = format!("{}/vaults/{}/{}", self.base_dir, vault_id, file_id);
        fs::remove_file(path).await?;
        Ok(())
    }

    pub async fn delete_vault(&self, vault_id: i64) -> Result<()> {
        let path = format!("{}/vaults/{}", self.base_dir, vault_id);
        fs::remove_dir_all(path).await?;
        Ok(())
    }

    pub async fn get_file_reader(&self, vault_id: i64, file_id: i64) -> Result<BufReader<File>> {
        let path = format!("{}/vaults/{}/{}", self.base_dir, vault_id, file_id);
        let file = fs::File::open(path).await?;
        Ok(BufReader::new(file))
    }
}
