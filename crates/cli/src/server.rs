use clap::Subcommand;
use eyre::{Result, WrapErr, eyre};
use kittynode_core::daemon::{DEFAULT_SERVER_PORT, ServerStatus, validate_server_port};
use std::collections::VecDeque;
use std::env;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, ErrorKind, Seek, SeekFrom, Write};
use std::path::Path;

const INTERNAL_RUN_SUBCOMMAND: &str = "__internal-run";

#[derive(Subcommand)]
pub enum ServerCommands {
    #[command(name = "start", about = "Start the Kittynode server")]
    Start {
        #[arg(
            long = "port",
            value_name = "PORT",
            help = "Port to bind the Kittynode server"
        )]
        port: Option<u16>,
    },
    #[command(name = "restart", about = "Restart the Kittynode server")]
    Restart {
        #[arg(
            long = "port",
            value_name = "PORT",
            help = "Port to bind the Kittynode server"
        )]
        port: Option<u16>,
    },
    #[command(name = "stop", about = "Stop the Kittynode server")]
    Stop,
    #[command(name = "status", about = "Show Kittynode server status")]
    Status,
    #[command(name = "logs", about = "Stream logs from the Kittynode server")]
    Logs {
        #[arg(
            long = "follow",
            short = 'f',
            help = "Follow log output until interrupted"
        )]
        follow: bool,
        #[arg(
            long = "tail",
            value_name = "LINES",
            help = "Number of lines to show from the end of the log"
        )]
        tail: Option<usize>,
    },
    #[command(name = "__internal-run", hide = true)]
    RunInternal {
        #[arg(
            long = "port",
            value_name = "PORT",
            help = "Port to bind the Kittynode server"
        )]
        port: Option<u16>,
        #[arg(
            long = "service-token",
            value_name = "TOKEN",
            hide = true,
            help = "Internal token used to bind the server to the parent process"
        )]
        service_token: Option<String>,
    },
}

impl ServerCommands {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Start { port } => start_server(port),
            Self::Restart { port } => restart_server(port),
            Self::Stop => stop_server(),
            Self::Status => server_status(),
            Self::Logs { follow, tail } => server_logs(follow, tail),
            Self::RunInternal {
                port,
                service_token,
            } => run_server(port, service_token).await,
        }
    }
}

fn start_server(port: Option<u16>) -> Result<()> {
    let binary = env::current_exe().wrap_err("Failed to locate kittynode binary")?;
    let port = port.map(validate_server_port).transpose()?;
    let status =
        kittynode_core::daemon::start_server(port, &binary, &["server", INTERNAL_RUN_SUBCOMMAND])?;
    println!("{status}");
    if let Ok(path) = kittynode_core::daemon::get_server_log_path() {
        println!("Logs: {}", path.display());
    }
    Ok(())
}

fn stop_server() -> Result<()> {
    let status = kittynode_core::daemon::stop_server()?;
    println!("{status}");
    Ok(())
}

fn restart_server(port: Option<u16>) -> Result<()> {
    let port = match port {
        Some(port) => Some(port),
        None => match kittynode_core::daemon::get_server_status()? {
            ServerStatus::Started { port, .. } | ServerStatus::AlreadyRunning { port, .. } => {
                Some(port)
            }
            _ => None,
        },
    };

    let status = kittynode_core::daemon::stop_server()?;
    println!("{status}");

    start_server(port)
}

fn server_status() -> Result<()> {
    match kittynode_core::daemon::get_server_status()? {
        ServerStatus::Started { pid, port } | ServerStatus::AlreadyRunning { pid, port } => {
            println!("Kittynode server running on port {port} (pid {pid})");
            if let Ok(path) = kittynode_core::daemon::get_server_log_path() {
                println!("Logs: {}", path.display());
            }
        }
        ServerStatus::Stopped { pid, port } => {
            println!("Kittynode server stopped (last seen pid {pid}, port {port})");
        }
        ServerStatus::NotRunning => {
            println!("Kittynode server is not running");
        }
    }
    Ok(())
}

fn server_logs(follow: bool, tail: Option<usize>) -> Result<()> {
    let tail = tail.filter(|value| *value > 0);
    let path = kittynode_core::daemon::get_server_log_path()
        .wrap_err("Failed to locate kittynode server logs")?;
    stream_log_file(&path, tail, follow)
        .wrap_err_with(|| format!("Failed to stream logs from {}", path.display()))?;
    Ok(())
}

fn stream_log_file(path: &Path, tail: Option<usize>, follow: bool) -> Result<()> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    stream_log_file_with_writer(path, tail, follow, &mut handle)
}

fn stream_log_file_with_writer(
    path: &Path,
    tail: Option<usize>,
    follow: bool,
    writer: &mut dyn Write,
) -> Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|err| eyre!("Failed to open log file: {err}"))?;

    let snapshot = collect_initial_log_output(
        BufReader::new(
            file.try_clone()
                .map_err(|err| eyre!("Failed to clone log file handle: {err}"))?,
        ),
        tail,
    )?;

    writer
        .write_all(snapshot.as_bytes())
        .map_err(|err| eyre!("Failed to write log output: {err}"))?;
    writer
        .flush()
        .map_err(|err| eyre!("Failed to flush stdout: {err}"))?;

    if !follow {
        return Ok(());
    }

    file.seek(SeekFrom::End(0))
        .map_err(|err| eyre!("Failed to seek log file: {err}"))?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    loop {
        match reader.read_line(&mut line) {
            Ok(0) => {
                line.clear();
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
            Ok(_) => {
                writer
                    .write_all(line.as_bytes())
                    .map_err(|err| eyre!("Failed to write log output: {err}"))?;
                writer
                    .flush()
                    .map_err(|err| eyre!("Failed to flush stdout: {err}"))?;
                line.clear();
            }
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) => return Err(eyre!("Failed while streaming logs: {err}")),
        }
    }
}

fn collect_initial_log_output<R: BufRead>(mut reader: R, tail: Option<usize>) -> Result<String> {
    if let Some(limit) = tail {
        read_tail_lines(&mut reader, limit)
    } else {
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .map_err(|err| eyre!("Failed to read kittynode server log file: {err}"))?;
        Ok(content)
    }
}

fn read_tail_lines<R: BufRead>(reader: &mut R, limit: usize) -> Result<String> {
    let mut buffer = VecDeque::with_capacity(limit);
    let mut line = String::new();
    loop {
        line.clear();
        let bytes = match reader.read_line(&mut line) {
            Ok(count) => count,
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) => {
                return Err(eyre!("Failed to read kittynode server log file: {err}"));
            }
        };
        if bytes == 0 {
            break;
        }
        if buffer.len() == limit {
            buffer.pop_front();
        }
        buffer.push_back(line.clone());
    }

    let mut collected = String::new();
    for entry in buffer {
        collected.push_str(&entry);
    }
    Ok(collected)
}

async fn run_server(port: Option<u16>, service_token: Option<String>) -> Result<()> {
    let port = validate_server_port(port.unwrap_or(DEFAULT_SERVER_PORT))?;
    let Some(_token) = service_token else {
        return Err(eyre!("server run invoked without token"));
    };
    kittynode_server::run_with_port(port).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{collect_initial_log_output, stream_log_file_with_writer};
    use std::io::Cursor;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn collect_initial_log_output_without_tail_returns_full_content() {
        let content = "first line\nsecond line\n";
        let result = collect_initial_log_output(Cursor::new(content.as_bytes()), None)
            .expect("reading snapshot without tail succeeds");

        assert_eq!(result, content);
    }

    #[test]
    fn collect_initial_log_output_with_tail_limits_to_requested_lines() {
        let content = "line1\nline2\nline3\n";
        let result = collect_initial_log_output(Cursor::new(content.as_bytes()), Some(2))
            .expect("reading snapshot with tail succeeds");

        assert_eq!(result, "line2\nline3\n");
    }

    #[test]
    fn stream_log_file_without_tail_writes_entire_contents() {
        let mut temp = NamedTempFile::new().expect("failed to create temp log file");
        writeln!(temp, "first line").expect("failed to write first line");
        writeln!(temp, "second line").expect("failed to write second line");
        temp.flush().expect("failed to flush log file");

        let mut buffer = Vec::new();
        stream_log_file_with_writer(temp.path(), None, false, &mut buffer)
            .expect("streaming log file without tail should succeed");

        let output = String::from_utf8(buffer).expect("log output should be utf8");
        assert_eq!(output, "first line\nsecond line\n");
    }

    #[test]
    fn stream_log_file_with_tail_limits_output() {
        let mut temp = NamedTempFile::new().expect("failed to create temp log file");
        writeln!(temp, "line 1").expect("failed to write line 1");
        writeln!(temp, "line 2").expect("failed to write line 2");
        writeln!(temp, "line 3").expect("failed to write line 3");
        temp.flush().expect("failed to flush log file");

        let mut buffer = Vec::new();
        stream_log_file_with_writer(temp.path(), Some(2), false, &mut buffer)
            .expect("streaming log file with tail should succeed");

        let output = String::from_utf8(buffer).expect("log output should be utf8");
        assert_eq!(output, "line 2\nline 3\n");
    }

    #[test]
    fn stream_log_file_tail_longer_than_log_outputs_all_lines() {
        let mut temp = NamedTempFile::new().expect("failed to create temp log file");
        writeln!(temp, "alpha").expect("failed to write alpha");
        writeln!(temp, "beta").expect("failed to write beta");
        temp.flush().expect("failed to flush log file");

        let mut buffer = Vec::new();
        stream_log_file_with_writer(temp.path(), Some(5), false, &mut buffer)
            .expect("streaming log file with large tail should succeed");

        let output = String::from_utf8(buffer).expect("log output should be utf8");
        assert_eq!(output, "alpha\nbeta\n");
    }
}
