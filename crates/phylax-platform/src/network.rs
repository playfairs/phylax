use phylax_common::{Error, Result};

pub struct NetworkInfo;

impl NetworkInfo {
    pub fn local_ip() -> Result<String> {
        Ok("127.0.0.1".to_string())
    }

    pub fn hostname() -> Result<String> {
        std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .ok_or_else(|| Error::PlatformError("Failed to get hostname".to_string()))
    }
}
