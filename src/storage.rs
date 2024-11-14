use std::env;

use anyhow::Result;
use aws_config::Region;
use aws_sdk_s3::{
    config::{Credentials, SharedCredentialsProvider},
    Client,
};

pub struct StorageEngine {
    client: Client,
}

impl StorageEngine {
    pub async fn new() -> Result<StorageEngine> {
        let access_key = env::var("S3_ACCESS_KEY")?;
        let secret_key = env::var("S3_SECRET_KEY")?;

        let cred = Credentials::new(access_key, secret_key, None, None, "loaded-from-custom-env");
        let config = aws_sdk_s3::config::Builder::new()
            .endpoint_url("http://127.0.0.1:9001")
            .region(Region::new("eu-central-1"))
            .force_path_style(true)
            .credentials_provider(cred)
            .build();

        let client = aws_sdk_s3::Client::from_conf(config);
        Ok(StorageEngine { client })
    }
}
