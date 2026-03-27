use anyhow::{Result, anyhow};

use crate::helpers::disk::DiskDevice;

#[derive(Default)]
pub struct Config {
    pub disk_device: Option<DiskDevice>,
    pub hardware_file: Option<Vec<u8>>,
    pub age_pk: Option<String>,
}

impl Config {
    pub fn get_disk_device(&self) -> Result<&DiskDevice> {
        Ok(self
            .disk_device
            .as_ref()
            .ok_or_else(|| anyhow!("Disk device has not been set"))?)
    }

    pub fn get_hardware_file(&self) -> Result<&Vec<u8>> {
        Ok(self
            .hardware_file
            .as_ref()
            .ok_or_else(|| anyhow!("Hardware file has not been set"))?)
    }

    pub fn get_age_key(&self) -> Result<&str> {
        Ok(self
            .age_pk
            .as_ref()
            .ok_or_else(|| anyhow!("Age key has not been set"))?)
    }
}
