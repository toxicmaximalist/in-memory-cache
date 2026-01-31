//! Entry point for the in-memory cache library.
//!
//! This file exists for cargo to have a default binary target.
//! For actual usage, run the server or client binaries:
//!
//! ```bash
//! cargo run --bin server
//! cargo run --bin client get <key>
//! ```

fn main() {
    eprintln!("This binary is not intended to be run directly.");
    eprintln!("Use one of the following commands:");
    eprintln!("  cargo run --bin server       - Start the cache server");
    eprintln!("  cargo run --bin client <cmd> - Run client commands");
    std::process::exit(1);
}
