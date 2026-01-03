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
    fn display_messages_are_human_readable() {
        let started = WebServiceStatus::Started { pid: 1, port: 3000 };
        assert_eq!(
            started.to_string(),
            "Kittynode web service started on port 3000 (pid 1)"
        );

        let running = WebServiceStatus::AlreadyRunning { pid: 2, port: 8080 };
        assert_eq!(
            running.to_string(),
            "Kittynode web service already running on port 8080 (pid 2)"
        );

        let stopped = WebServiceStatus::Stopped { pid: 3, port: 9000 };
        assert_eq!(
            stopped.to_string(),
            "Kittynode web service stopped on port 9000 (pid 3)"
        );

        assert_eq!(
            WebServiceStatus::NotRunning.to_string(),
            "Kittynode web service is not running"
        );
    }
}
