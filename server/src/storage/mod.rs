use std::{io::Read, path::PathBuf};

use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader, BufWriter},
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
    pub async fn write_new(path: PathBuf, info: ChunkInfo) -> Result<Chunk> {
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

    pub async fn read_from(path: &PathBuf) -> Result<Chunk> {
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
            file,
            path: path.to_owned(),
            len: chunk_len,
        })
    }

    pub fn can_take_more_files(&self) -> bool {
        // if file_size % PAGE_SIZE != 0 {
        //     file_size += PAGE_SIZE - file_size % PAGE_SIZE;
        // }
        // self.len + file_size + PAGE_SIZE <= CHUNK_MAX_SIZE
        self.len < CHUNK_MAX_SIZE
    }

    pub async fn write_file(
        &mut self,
        file_size: usize,
        file_id: u64,
        mut reader: BufReader<File>,
    ) -> Result<u64> {
        if !self.can_take_more_files() {
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

        let cursor = self.file.stream_position().await?;
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
        drop(writer);

        self.len = self.file.metadata().await?.len().try_into()?;
        Ok(cursor)
    }
}

fn u32_to_bytes(value: u32) -> [u8; 4] {
    value.to_be_bytes()
}

fn pad_vec_to_page(value: &mut Vec<u8>) {
    value.resize(4096, 0);
}

pub struct VaultStore {
    pub vault_id: u64,
    pub chunks: Vec<Chunk>,
    pub base_dir: PathBuf,
    pub vault_dir: PathBuf,
}

impl VaultStore {
    pub async fn new(vault_id: u64, base_dir: PathBuf) -> Result<VaultStore> {
        let vault_dir = base_dir.join(format!("vault_{vault_id}"));
        fs::create_dir_all(&vault_dir).await?;

        // list all chunks in that dir
        let mut entries = fs::read_dir(&vault_dir).await?;
        let mut chunks = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let file_type = entry.file_type().await?;
            if !file_type.is_file() {
                continue;
            }

            let chunk = Chunk::read_from(&entry.path()).await?;
            chunks.push(chunk);
        }

        if chunks.is_empty() {
            let path = vault_dir.join("chunk-0.bin");
            let chunk = Chunk::write_new(
                path,
                ChunkInfo {
                    id: 0,
                    created_at: Utc::now(),
                },
            )
            .await?;

            chunks.push(chunk);
        }

        chunks.sort_by(|a, b| b.len.cmp(&a.len));
        Ok(VaultStore {
            vault_id,
            base_dir,
            vault_dir,
            chunks,
        })
    }

    pub async fn write_file(
        &mut self,
        file_size: usize,
        file_id: u64,
        reader: BufReader<File>,
    ) -> anyhow::Result<u64> {
        let last_chunk = self
            .chunks
            .last_mut()
            .expect("why are there no chunks LO!L!L!L!L (im going insane)");

        if last_chunk.can_take_more_files() {
            let cursor = last_chunk.write_file(file_size, file_id, reader).await?;
            Ok(cursor)
        } else {
            let id = self.chunks.len() as u64;
            let path = self.vault_dir.join(format!("chunk-{id}.bin"));
            let mut chunk = Chunk::write_new(
                path,
                ChunkInfo {
                    id,
                    created_at: Utc::now(),
                },
            )
            .await?;
            let cursor = chunk.write_file(file_size, file_id, reader).await?;
            self.chunks.push(chunk);
            Ok(cursor)
        }
    }
}
