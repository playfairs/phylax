use phylax_common::{Error, Result};

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub pid: Option<u32>,
    pub status: String,
}

pub struct ServiceManager;

impl ServiceManager {
    pub fn list_services(&self) -> Result<Vec<ServiceInfo>> {
        Ok(Vec::new())
    }

    pub fn get_service(&self, name: &str) -> Result<Option<ServiceInfo>> {
        Ok(None)
    }
}
