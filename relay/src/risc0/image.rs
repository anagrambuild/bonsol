use std::path::PathBuf;

use {
    anyhow::Result,
    bytes::Bytes,
    risc0_binfmt::{MemoryImage, Program},
    risc0_zkvm::{GUEST_MAX_MEM, PAGE_SIZE},
    tokio::fs::read,
};

pub struct Image {
    pub id: String,
    pub data: Option<Program>,
    pub size: u64,
    pub path: PathBuf,
    pub last_used: u64,
}

impl Image {
    fn load_elf(elf: &[u8]) -> Result<Program> {
        let program = Program::load_elf(elf, GUEST_MAX_MEM as u32)?;
        Ok(program)
    }

    fn mem_img(program: &Program) -> Result<MemoryImage> {
        let image = MemoryImage::new(program, PAGE_SIZE as u32)?;
        Ok(image)
    }

    pub fn from_bytes(bytes: Bytes) -> Result<Image> {
        let program = Image::load_elf(&bytes)?;
        let img = Image::mem_img(&program)?;
        Ok(Image {
            id: img.compute_id().map(|d| d.to_string())?,
            data: Some(program),
            size: img.pages.len() as u64 * PAGE_SIZE as u64,
            path: PathBuf::new(),
            last_used: 0,
        })
    }

    pub async fn new(path: PathBuf) -> Result<Image> {
        let data = read(&path).await?;
        let program = Image::load_elf(&data)?;
        let img = Image::mem_img(&program)?;

        Ok(Image {
            id: img.compute_id().map(|d| d.to_string())?,
            data: Some(program),
            size: img.pages.len() as u64 * PAGE_SIZE as u64,
            path,
            last_used: 0,
        })
    }

    pub fn compress(&mut self) {
        self.data = None;
    }

    pub async fn load(&mut self) -> Result<()> {
        if self.data.is_some() {
            return Ok(());
        }
        let data = read(&self.path).await?;
        let program = Image::load_elf(&data)?;
        self.data = Some(program);
        Ok(())
    }

    pub fn get_memory_image(&self) -> Result<MemoryImage> {
        let program = self
            .data
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No data"))?;
        let image = Image::mem_img(program)?;
        Ok(image)
    }
}
