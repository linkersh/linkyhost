use std::{io::Read, path::PathBuf, str::FromStr};

use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
};

const CHUNK_HEADER: [u8; 8] = [102, 212, 109, 199, 255, 10, 45, 3];
const CHUNK_MAX_SIZE: usize = 1024 * 1024 * 1024 * 2; // 2 Gibibyte
const PAGE_SIZE: usize = 4096;

pub struct ChunkDataReader {
    reader: BufReader<File>,
}

impl ChunkDataReader {
    pub fn new(reader: BufReader<File>) -> ChunkDataReader {
        ChunkDataReader { reader }
    }

    pub async fn read_page(&mut self) -> anyhow::Result<[u8; PAGE_SIZE]> {
        let mut page = [0u8; PAGE_SIZE];
        self.reader.read_exact(&mut page).await?;
        Ok(page)
    }

    pub fn into_inner(self) -> BufReader<File> {
        self.reader
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkPartHeader {
    pub len: usize,
    pub is_terminal: bool,
    pub file_id: u64,
}

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
        let mut page = Vec::with_capacity(PAGE_SIZE);

        page.append(&mut CHUNK_HEADER.to_vec());

        let mut info_bytes = bincode::serialize(&info)?;
        let info_len = info_bytes.len();
        let mut info_len_bytes = u32_to_bytes(info_len.try_into()?).to_vec();

        page.append(&mut info_len_bytes);
        page.append(&mut info_bytes);
        page.resize(PAGE_SIZE, 0);

        assert!(page.len() == PAGE_SIZE);

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

    pub fn file_fits(&self, mut file_size: usize) -> bool {
        if file_size % PAGE_SIZE != 0 {
            file_size += PAGE_SIZE - file_size % PAGE_SIZE;
        }
        self.len + file_size + PAGE_SIZE <= CHUNK_MAX_SIZE
    }

    pub async fn write_file(
        &mut self,
        file_size: usize,
        file_id: u64,
        mut reader: BufReader<File>,
    ) -> Result<()> {
        if !self.file_fits(file_size) {
            return Err(Error::msg("this is definitely a bug, this function should not be called if the file doesnt not fully fit in the chunk."));
        }

        let chunk_part_header = ChunkPartHeader {
            file_id,
            len: file_size,
            is_terminal: true,
        };

        let mut header_bytes = bincode::serialize(&chunk_part_header)?;

        let header_len: u32 = header_bytes.len().try_into()?;
        let mut header_len_bytes = header_len.to_be_bytes().to_vec();

        let mut page = Vec::with_capacity(PAGE_SIZE);
        page.append(&mut header_len_bytes);
        page.append(&mut header_bytes);

        assert!(page.len() <= PAGE_SIZE);
        pad_vec_to_page(&mut page);

        let mut writer = BufWriter::new(&mut self.file);
        writer.write_all(&page).await?;

        let mut buf = Vec::with_capacity(PAGE_SIZE * 2);
        let mut total_bytes_written = 0;
        loop {
            buf.clear();

            let n = reader.read_buf(&mut buf).await?;
            if n == 0 {
                break;
            }

            writer.write_all(&buf).await?;

            total_bytes_written += n;
            reader.consume(n);
        }

        if total_bytes_written % PAGE_SIZE != 0 {
            let zeros = vec![0u8; PAGE_SIZE - file_size % PAGE_SIZE];
            writer.write_all(&zeros).await?;
        }

        writer.flush().await?;
        Ok(())
    }
}

fn u32_to_bytes(value: u32) -> [u8; 4] {
    value.to_be_bytes()
}

fn pad_vec_to_page(value: &mut Vec<u8>) {
    value.resize(4096, 0);
}
