use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiskDevices {
    pub blockdevices: Vec<DiskDevice>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiskDevice {
    pub name: String,
    size: String,
    model: Option<String>,
    mountpoint: Option<String>,
}

impl DiskDevice {
    pub fn get_info(&self) -> String {
        format!(
            "{} (size: {} / model: {:?} / mountpoint: {:?})",
            self.name, self.size, self.model, self.mountpoint
        )
    }
}
