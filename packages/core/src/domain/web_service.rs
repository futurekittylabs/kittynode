use std::fmt;

pub const DEFAULT_WEB_PORT: u16 = 3000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebServiceStatus {
    /// A new web service process was spawned successfully.
    Started { pid: u32, port: u16 },
    /// An existing web service is already running and matches the tracked state.
    AlreadyRunning { pid: u32, port: u16 },
    /// A previously tracked web service was stopped.
    Stopped { pid: u32, port: u16 },
    /// No active web service process could be found for the tracked state.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_started_formats_message_with_port_and_pid() {
        let status = WebServiceStatus::Started {
            pid: 12345,
            port: 3000,
        };
        assert_eq!(
            status.to_string(),
            "Kittynode web service started on port 3000 (pid 12345)"
        );
    }

    #[test]
    fn display_already_running_formats_message_with_port_and_pid() {
        let status = WebServiceStatus::AlreadyRunning {
            pid: 67890,
            port: 8080,
        };
        assert_eq!(
            status.to_string(),
            "Kittynode web service already running on port 8080 (pid 67890)"
        );
    }

    #[test]
    fn display_stopped_formats_message_with_port_and_pid() {
        let status = WebServiceStatus::Stopped {
            pid: 99999,
            port: 4000,
        };
        assert_eq!(
            status.to_string(),
            "Kittynode web service stopped on port 4000 (pid 99999)"
        );
    }

    #[test]
    fn display_not_running_formats_simple_message() {
        let status = WebServiceStatus::NotRunning;
        assert_eq!(status.to_string(), "Kittynode web service is not running");
    }
}
