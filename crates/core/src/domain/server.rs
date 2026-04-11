use std::fmt;

pub const DEFAULT_SERVER_PORT: u16 = 3000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServerStatus {
    /// A new server process was spawned successfully.
    Started { pid: u32, port: u16 },
    /// An existing server is already running and matches the tracked state.
    AlreadyRunning { pid: u32, port: u16 },
    /// A previously tracked server was stopped.
    Stopped { pid: u32, port: u16 },
    /// No active server process could be found for the tracked state.
    NotRunning,
}

impl fmt::Display for ServerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerStatus::Started { pid, port } => {
                write!(f, "Kittynode server started on port {port} (pid {pid})")
            }
            ServerStatus::AlreadyRunning { pid, port } => {
                write!(
                    f,
                    "Kittynode server already running on port {port} (pid {pid})"
                )
            }
            ServerStatus::Stopped { pid, port } => {
                write!(f, "Kittynode server stopped on port {port} (pid {pid})")
            }
            ServerStatus::NotRunning => write!(f, "Kittynode server is not running"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages_are_human_readable() {
        let started = ServerStatus::Started { pid: 1, port: 3000 };
        assert_eq!(
            started.to_string(),
            "Kittynode server started on port 3000 (pid 1)"
        );

        let running = ServerStatus::AlreadyRunning { pid: 2, port: 8080 };
        assert_eq!(
            running.to_string(),
            "Kittynode server already running on port 8080 (pid 2)"
        );

        let stopped = ServerStatus::Stopped { pid: 3, port: 9000 };
        assert_eq!(
            stopped.to_string(),
            "Kittynode server stopped on port 9000 (pid 3)"
        );

        assert_eq!(
            ServerStatus::NotRunning.to_string(),
            "Kittynode server is not running"
        );
    }
}
