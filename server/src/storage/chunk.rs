use std::{io::Read, path::PathBuf, str::FromStr};

use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
};

const CHUNK_HEADER: [u8; 8] = [102, 212, 109, 199, 255, 10, 45, 3];
const CHUNK_MAX_SIZE: usize = 1024 * 1024 * 1024 * 2; // 2 Gibibyte

pub struct ChunkDataReader {
    reader: BufReader<File>,
}

impl ChunkDataReader {
    pub fn new(reader: BufReader<File>) -> ChunkDataReader {
        ChunkDataReader { reader }
    }

    pub async fn read_page(&mut self) -> anyhow::Result<[u8; 4096]> {
        let mut page = [0u8; 4096];
        self.reader.read_exact(&mut page).await?;
        Ok(page)
    }

    pub fn into_inner(self) -> BufReader<File> {
        self.reader
    }
}

pub struct ChunkPart {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkInfo {
    pub id: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct Chunk {
    pub info: ChunkInfo,
    pub path: PathBuf,
    pub file: File,
    pub len: usize,
}

impl Chunk {
    pub async fn write_new(path: PathBuf, info: ChunkInfo) -> anyhow::Result<Chunk> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .await?;
        let mut writer = BufWriter::new(file);
        let mut page = Vec::with_capacity(4096);

        page.append(&mut CHUNK_HEADER.to_vec());

        let mut info_bytes = bincode::serialize(&info)?;
        let info_len = info_bytes.len();
        let mut info_len_bytes = u32_to_bytes(info_len.try_into()?).to_vec();

        page.append(&mut info_len_bytes);
        page.append(&mut info_bytes);
        page.resize(4096, 0);

        assert!(page.len() == 4096);

        writer.write_all(&page).await?;
        writer.flush().await?;

        let file = writer.into_inner();
        let chunk_len: usize = file.metadata().await?.len().try_into()?;

        Ok(Chunk {
            file,
            info,
            path,
            len: chunk_len,
        })
    }

    pub async fn read_from(path: &str) -> Result<Chunk> {
        let path = PathBuf::from_str(path)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .await?;

        let reader = BufReader::new(file);
        let mut chunk_data_reader = ChunkDataReader::new(reader);

        let page = chunk_data_reader.read_page().await?;
        let header = &page[0..8];

        if header != CHUNK_HEADER {
            return Err(Error::msg("chunk header is invalid"));
        }

        let info_len: [u8; 4] = page[8..12].try_into()?;
        let info_len: usize = u32::from_be_bytes(info_len).try_into()?;

        let info_bytes = &page[12..(info_len + 12)];
        let info: ChunkInfo = bincode::deserialize(info_bytes)?;

        let file = chunk_data_reader.into_inner().into_inner();
        let chunk_len: usize = file.metadata().await?.len().try_into()?;

        Ok(Chunk {
            info,
            path,
            file,
            len: chunk_len,
        })
    }

    pub async fn write_file(&self, file_size: usize, bytes: &[u8]) -> Result<()> {
        if file_size + self.len > CHUNK_MAX_SIZE {
            // well fuck.
            return Err(Error::msg("well fuck"));
        }

        todo!()
    }
}

fn bytes_to_u32(bytes: [u8; 4]) -> u32 {
    u32::from_be_bytes(bytes)
}

fn u32_to_bytes(value: u32) -> [u8; 4] {
    value.to_be_bytes()
}
