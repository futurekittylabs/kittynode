use eyre::{Result, eyre};
use kittynode_core::api::DEFAULT_WEB_PORT;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let port = parse_port()?;
    kittynode_web::run_with_port(port).await
}

fn parse_port() -> Result<u16> {
    let mut port = DEFAULT_WEB_PORT;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--port=") {
            port = parse_port_value(value)?;
        } else if arg == "--port" {
            let value = args
                .next()
                .ok_or_else(|| eyre!("--port requires a value"))?;
            port = parse_port_value(&value)?;
        } else {
            return Err(eyre!("Unknown argument: {arg}"));
        }
    }

    Ok(port)
}

fn parse_port_value(value: &str) -> Result<u16> {
    let port: u16 = value
        .parse()
        .map_err(|_| eyre!("Invalid port value: {value}"))?;
    if port == 0 {
        return Err(eyre!("Port must be greater than zero"));
    }
    Ok(port)
}
