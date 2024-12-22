use std::net::SocketAddr;
use base64::{engine::general_purpose, Engine as _};

use clap::Parser;
use configuration::{get_configuration, Configuration};
use logger::setup_logging;
use tokio::io::AsyncWriteExt;


mod logger;
mod configuration;

#[derive(Parser)]
#[clap(author, version, about = env!("CARGO_PKG_DESCRIPTION"), long_about = None)]
struct Args {
    #[clap(short, long)]
    config_folder: Option<String>,
}

// New struct to hold authentication credentials
#[derive(Debug, Clone)]
struct AuthConfig {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let args = Args::parse();

    let config = get_configuration(&args.config_folder.unwrap_or("config".to_string()))?;

    let default_log_lvl = "debug".to_string();

    setup_logging(config.application.log_lvl.as_ref().unwrap_or(&default_log_lvl));

    run(config).await?;

    Ok(())
}

#[tracing::instrument(name = "run", skip(config))]
pub async fn run(config: Configuration) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Config: {:?}", &config);

    let ip: String = config.application.host.unwrap_or("0.0.0.0".to_string());
    let port: u16 = config.application.port.unwrap_or(7547);
    let addr = format!("{}:{}", ip, port).parse::<SocketAddr>()?;

    let timeout = config.application.timeout.unwrap_or(30);
    
    // Extract authentication credentials from config
    let auth_config = config.auth.clone().map(|auth| AuthConfig {
        username: auth.username.unwrap_or_else(|| "admin".to_string()),
        password: auth.password.unwrap_or_else(|| "password".to_string()),
    });

    tracing::info!("Listening on {}", addr);
    tracing::info!("Timeout: {} seconds", timeout);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    accept_connections(listener, timeout, auth_config).await?;

    Ok(())
}

#[tracing::instrument(name = "validate_basic_auth", skip(auth_config))]
fn validate_basic_auth(auth_header: &str, auth_config: &Option<AuthConfig>) -> bool {
    // If no auth config is provided, allow all connections
    let Some(auth) = auth_config else {
        return true;
    };

    // Extract the header value, trimming "Authorization: " if present
    let auth_header = auth_header.trim_start_matches("Authorization: ").trim();

    // Check if the header starts with "Basic "
    if !auth_header.starts_with("Basic ") {
        tracing::warn!("Invalid Authorization header format");
        return false;
    }

    // Decode the base64 encoded credentials
    let encoded_credentials = &auth_header[6..];
    match general_purpose::STANDARD.decode(encoded_credentials) {
        Ok(decoded) => {
            let credentials = match String::from_utf8(decoded) {
                Ok(cred) => cred,
                Err(_) => {
                    tracing::warn!("Failed to parse credentials");
                    return false;
                }
            };

            // Split credentials into username and password
            let parts: Vec<&str> = credentials.split(':').collect();
            if parts.len() != 2 {
                tracing::warn!("Invalid credentials format");
                return false;
            }

            // Log the attempted username (but not the password for security)
            tracing::info!("Auth attempt for username: {}", parts[0]);

            // Compare with stored credentials
            parts[0] == auth.username && parts[1] == auth.password
        }
        Err(_) => {
            tracing::warn!("Failed to decode Base64 credentials");
            false
        }
    }
}

#[tracing::instrument(name = "accept_connections", skip(listener, timeout, auth_config))]
async fn accept_connections(
    listener: tokio::net::TcpListener, 
    timeout: u64, 
    auth_config: Option<AuthConfig>
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        match listener.accept().await {
            Ok((mut socket, peer_addr)) => {
                tracing::info!("Accepted connection from {}", peer_addr);
                
                let auth_config = auth_config.clone();
                
                tokio::spawn(async move {
                    let mut buffer = [0; 2048];
                    
                    let timeout = tokio::time::Duration::from_secs(timeout);
                    
                    loop {
                        match tokio::time::timeout(timeout, socket.readable()).await {
                            Ok(Ok(())) => {
                                match socket.try_read(&mut buffer) {
                                    Ok(0) => {
                                        tracing::info!("Connection closed by {}", peer_addr);
                                        break;
                                    }
                                    Ok(n) => {
                                        if let Ok(data) = String::from_utf8(buffer[..n].to_vec()) {
                                            // Check for Authorization header
                                            let lines: Vec<&str> = data.lines().collect();
                                            let mut is_authorized = false;
                                            let mut auth_header = None;

                                            for line in &lines {
                                                if line.starts_with("Authorization: ") {
                                                    // Log the full Authorization header
                                                    let base64_part = line.trim_start_matches("Authorization: Basic ");
                                                    let decoded = general_purpose::STANDARD.decode(base64_part).unwrap();
                                                    let decoded_str = String::from_utf8(decoded).unwrap();
                                                    tracing::info!("Full Authorization header from {}: {}", peer_addr, decoded_str);
                                                    auth_header = Some(line);
                                                    break;
                                                }
                                            }

                                            if let Some(header) = auth_header {
                                                is_authorized = validate_basic_auth(header, &auth_config);
                                            }

                                            if !is_authorized {
                                                // Send 401 Unauthorized response
                                                let _ = socket.write_all(
                                                    b"HTTP/1.1 401 Unauthorized\r\n\
                                                    WWW-Authenticate: Basic realm=\"Restricted\"\r\n\
                                                    Content-Length: 0\r\n\r\n"
                                                ).await;
                                                tracing::warn!("Unauthorized access attempt from {}", peer_addr);
                                                break;
                                            }

                                            tracing::info!("Received from {}: {}", peer_addr, data.trim());
                                        } else {
                                            tracing::warn!("Received non-UTF8 data from {}", peer_addr);
                                        }
                                    }
                                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                        continue;
                                    }
                                    Err(e) => {
                                        tracing::error!("Error reading from {}: {}", peer_addr, e);
                                        break;
                                    }
                                }
                            }
                            Ok(Err(e)) => {
                                tracing::error!("Socket readable error from {}: {}", peer_addr, e);
                                break;
                            }
                            Err(_) => {
                                let _ = socket.try_write(b"HTTP/1.1 408 Request Timeout\r\nContent-Length: 0\r\n\r\n");
                                tracing::info!("Connection timed out for {}", peer_addr);
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                tracing::error!("Failed to accept connection: {}", e);
            }
        }
    }
}