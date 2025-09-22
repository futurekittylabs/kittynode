use std::fmt;

pub const DEFAULT_WEB_PORT: u16 = 3000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebServiceStatus {
    Started { pid: u32, port: u16 },
    AlreadyRunning { pid: u32, port: u16 },
    Stopped { pid: u32, port: u16 },
    NotRunning,
}

impl fmt::Display for WebServiceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebServiceStatus::Started { pid, port } => {
                write!(
                    f,
                    "Kittynode web service started on port {port} (pid {pid})"
                )
            }
            WebServiceStatus::AlreadyRunning { pid, port } => {
                write!(
                    f,
                    "Kittynode web service already running on port {port} (pid {pid})"
                )
            }
            WebServiceStatus::Stopped { pid, port } => {
                write!(
                    f,
                    "Kittynode web service stopped on port {port} (pid {pid})"
                )
            }
            WebServiceStatus::NotRunning => write!(f, "Kittynode web service is not running"),
        }
    }
}
