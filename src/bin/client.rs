//! In-memory cache client.
//!
//! This binary provides a CLI for interacting with a running cache server.

use bytes::BytesMut;
use clap::Parser;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use in_memory_cache::cli::{Cli, ClientCommand};

/// Default server address.
const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3000;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let addr = format!("{}:{}", DEFAULT_HOST, DEFAULT_PORT);
    let mut stream = match TcpStream::connect(&addr).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to connect to server at {}: {}", addr, e);
            eprintln!("Make sure the server is running with: cargo run --bin server");
            std::process::exit(1);
        }
    };

    match args.command {
        ClientCommand::Set { key, value } => {
            // Send: set <key> <value>
            let cmd = format!("set {} {}", key, value);
            stream.write_all(cmd.as_bytes()).await?;

            let mut buf = BytesMut::with_capacity(1024);
            let _ = stream.read_buf(&mut buf).await?;

            match std::str::from_utf8(&buf) {
                Ok("r Ok") => println!("Updated key '{}'", key),
                Ok("Ok") => println!("Set key '{}'", key),
                Ok(resp) if resp.starts_with("ERR") => {
                    eprintln!("Error: {}", resp);
                    std::process::exit(1);
                }
                Ok(resp) => println!("Response: {}", resp),
                Err(e) => {
                    eprintln!("Failed to parse response: {}", e);
                    std::process::exit(1);
                }
            }
        }

        ClientCommand::Get { key } => {
            // Send: get <key>
            let cmd = format!("get {}", key);
            stream.write_all(cmd.as_bytes()).await?;

            let mut buf = BytesMut::with_capacity(1024);
            let _ = stream.read_buf(&mut buf).await?;

            match std::str::from_utf8(&buf) {
                Ok("") => println!("Key '{}' not found", key),
                Ok(resp) if resp.starts_with("ERR") => {
                    eprintln!("Error: {}", resp);
                    std::process::exit(1);
                }
                Ok(value) => println!("{}", value),
                Err(e) => {
                    eprintln!("Failed to parse response: {}", e);
                    std::process::exit(1);
                }
            }
        }

        ClientCommand::Delete { key } => {
            // Send: delete <key>
            let cmd = format!("delete {}", key);
            stream.write_all(cmd.as_bytes()).await?;

            let mut buf = BytesMut::with_capacity(1024);
            let _ = stream.read_buf(&mut buf).await?;

            match std::str::from_utf8(&buf) {
                Ok("Ok") => println!("Deleted key '{}'", key),
                Ok("") => println!("Key '{}' not found", key),
                Ok(resp) => println!("Response: {}", resp),
                Err(e) => {
                    eprintln!("Failed to parse response: {}", e);
                    std::process::exit(1);
                }
            }
        }

        ClientCommand::Ping => {
            stream.write_all(b"ping").await?;

            let mut buf = BytesMut::with_capacity(1024);
            let _ = stream.read_buf(&mut buf).await?;

            match std::str::from_utf8(&buf) {
                Ok("PONG") => println!("PONG"),
                Ok(resp) => println!("Response: {}", resp),
                Err(e) => {
                    eprintln!("Failed to parse response: {}", e);
                    std::process::exit(1);
                }
            }
        }

        ClientCommand::Stats => {
            stream.write_all(b"stats").await?;

            let mut buf = BytesMut::with_capacity(1024);
            let _ = stream.read_buf(&mut buf).await?;

            match std::str::from_utf8(&buf) {
                Ok(resp) => {
                    println!("Cache Statistics:");
                    for part in resp.split_whitespace() {
                        if let Some((key, value)) = part.split_once(':') {
                            println!("  {}: {}", key, value);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse response: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
