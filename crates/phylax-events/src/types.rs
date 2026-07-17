use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationEvent {
    pub username: String,
    pub method: AuthMethod,
    pub success: bool,
    pub source_ip: Option<IpAddr>,
    pub terminal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    Password,
    SshKey,
    Sso,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvent {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub executable: String,
    pub arguments: Vec<String>,
    pub user: Option<String>,
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemEvent {
    pub path: String,
    pub file_type: FileType,
    pub size: Option<u64>,
    pub permissions: Option<u32>,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    File,
    Directory,
    Symlink,
    Device,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    pub source_ip: IpAddr,
    pub source_port: u16,
    pub destination_ip: IpAddr,
    pub destination_port: u16,
    pub protocol: NetworkProtocol,
    pub direction: NetworkDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkProtocol {
    Tcp,
    Udp,
    Icmp,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkDirection {
    Inbound,
    Outbound,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeEvent {
    pub from_user: Option<String>,
    pub to_user: String,
    pub method: PrivilegeMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrivilegeMethod {
    Sudo,
    Su,
    Direct,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceEvent {
    pub mechanism: PersistenceMechanism,
    pub path: Option<String>,
    pub name: String,
    pub action: PersistenceAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PersistenceMechanism {
    LaunchAgent,
    LaunchDaemon,
    SystemdService,
    CronJob,
    RegistryKey,
    StartupFolder,
    InitScript,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PersistenceAction {
    Created,
    Modified,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationEvent {
    pub path: String,
    pub setting: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEvent {
    pub name: String,
    pub service_type: ServiceType,
    pub status: ServiceStatus,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceType {
    Systemd,
    Launchd,
    WindowsService,
    Init,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Started,
    Stopped,
    Running,
    Failed,
    Unknown,
}
