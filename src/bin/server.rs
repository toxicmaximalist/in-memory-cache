//! In-memory cache server.
//!
//! This binary runs a TCP server that accepts cache commands from clients.

use bytes::BytesMut;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    signal,
};

use in_memory_cache::{buffer_to_array, Cache, CacheConfig, Command};

/// Server configuration with defaults.
struct ServerConfig {
    host: String,
    port: u16,
    max_capacity: Option<usize>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            max_capacity: Some(10_000),
        }
    }
}

/// Entry point for the cache server.
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::default();

    // Build cache configuration
    let cache_config = CacheConfig::new()
        .max_capacity(config.max_capacity.unwrap_or(0))
        .build();

    // Create the shared cache
    let cache = Arc::new(Cache::new(cache_config));

    // Bind the listener
    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;

    println!("Cache server listening on {}", addr);
    println!("Max capacity: {:?}", config.max_capacity);

    // Spawn a task to handle graceful shutdown
    let shutdown_cache = Arc::clone(&cache);
    tokio::spawn(async move {
        if let Ok(()) = signal::ctrl_c().await {
            println!("\nShutting down...");
            let stats = shutdown_cache.stats();
            println!(
                "Final stats: hits={}, misses={}, size={}",
                stats.hits, stats.misses, stats.size
            );
        }
    });

    // Accept connections in a loop
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("Connection from {}", addr);

                // Clone the cache handle for this connection
                let cache = Arc::clone(&cache);

                // Spawn a task to handle this connection
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(socket, cache).await {
                        eprintln!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

/// Handle a single client connection.
async fn handle_connection(
    mut socket: TcpStream,
    cache: Arc<Cache>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = BytesMut::with_capacity(1024);

    // Read the request
    let n = socket.read_buf(&mut buf).await?;
    if n == 0 {
        return Ok(()); // Connection closed
    }

    // Parse the command
    let attrs = buffer_to_array(&mut buf);

    if attrs.is_empty() {
        socket.write_all(b"ERR empty command").await?;
        return Ok(());
    }

    let command = Command::get(&attrs[0]);

    // Process the command
    let response = process_command(command, &attrs, &cache).await;

    // Send the response
    socket.write_all(response.as_bytes()).await?;

    Ok(())
}

/// Process a cache command and return the response.
async fn process_command(command: Command, attrs: &[String], cache: &Cache) -> String {
    match command {
        Command::Get => {
            if attrs.len() < 2 {
                return "ERR missing key argument".to_string();
            }

            let key = &attrs[1];
            match cache.get(key) {
                Some(value) => {
                    // Convert bytes to string for response
                    match std::str::from_utf8(&value) {
                        Ok(s) => s.to_string(),
                        Err(_) => format!("(binary data: {} bytes)", value.len()),
                    }
                }
                None => String::new(), // Empty string for not found (legacy behavior)
            }
        }

        Command::Set => {
            if attrs.len() < 3 {
                return "ERR missing key or value argument".to_string();
            }

            let key = &attrs[1];
            let value = &attrs[2];

            let existed = cache.contains(key);
            cache.set(key.clone(), value.clone());

            if existed {
                "r Ok".to_string() // Replaced
            } else {
                "Ok".to_string() // New key
            }
        }

        Command::Delete => {
            if attrs.len() < 2 {
                return "ERR missing key argument".to_string();
            }

            let key = &attrs[1];
            if cache.delete(key) {
                "Ok".to_string()
            } else {
                String::new() // Not found
            }
        }

        Command::Ping => "PONG".to_string(),

        Command::Stats => {
            let stats = cache.stats();
            format!(
                "hits:{} misses:{} size:{} hit_rate:{:.1}%",
                stats.hits, stats.misses, stats.size, stats.hit_rate
            )
        }

        Command::Invalid => {
            format!(
                "ERR unknown command '{}'",
                attrs.first().unwrap_or(&String::new())
            )
        }
    }
}
