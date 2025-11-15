//! Static file server for wallet connection page
//!
//! Serves the Leptos WASM app from the dist/ directory on port 8080

use std::fs;
use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).expect("Failed to bind to port 8080");

    println!("Wallet connection server running at http://{}", addr);
    println!("Serving from dist/ directory");
    println!("Press Ctrl+C to stop\n");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => eprintln!("Connection error: {}", e),
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = match buf_reader.lines().next() {
        Some(Ok(line)) => line,
        _ => {
            eprintln!("Failed to read request line");
            return;
        }
    };

    // Parse the request path and query string
    let full_path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/");
    
    let (path, _query) = match full_path.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (full_path, None),
    };

    // Map paths to files in dist/
    let file_path = if path == "/" || path.is_empty() {
        PathBuf::from("dist/index.html")
    } else {
        // For other paths, try to serve from dist/ directory
        let mut dist_path = PathBuf::from("dist");
        dist_path.push(path.strip_prefix('/').unwrap_or(path));
        
        // If it's a directory or doesn't exist, serve index.html (for client-side routing)
        if dist_path.is_dir() || !dist_path.exists() {
            PathBuf::from("dist/index.html")
        } else {
            dist_path
        }
    };

    // Determine content type
    let content_type = if file_path.extension().and_then(|s| s.to_str()) == Some("html") {
        "text/html; charset=utf-8"
    } else if file_path.extension().and_then(|s| s.to_str()) == Some("css") {
        "text/css"
    } else if file_path.extension().and_then(|s| s.to_str()) == Some("js") {
        "application/javascript"
    } else if file_path.extension().and_then(|s| s.to_str()) == Some("wasm") {
        "application/wasm"
    } else if file_path.extension().and_then(|s| s.to_str()) == Some("json") {
        "application/json"
    } else {
        "application/octet-stream"
    };

    // Read file contents
    let (file_contents, final_content_type) = if file_path.exists() {
        match fs::read(&file_path) {
            Ok(contents) => (contents, content_type),
            Err(_) => {
                // Fallback to index.html
                let index_path = Path::new("dist/index.html");
                match fs::read(index_path) {
                    Ok(contents) => (contents, "text/html; charset=utf-8"),
                    Err(_) => {
                        eprintln!("File not found: {}", file_path.display());
                        let error_msg = b"<!DOCTYPE html><html><body><h1>Error: File not found</h1></body></html>".to_vec();
                        (error_msg, "text/html")
                    }
                }
            }
        }
    } else {
        // File doesn't exist, serve index.html for client-side routing
        let index_path = Path::new("dist/index.html");
        match fs::read(index_path) {
            Ok(contents) => (contents, "text/html; charset=utf-8"),
            Err(_) => {
                eprintln!("Index.html not found");
                let error_msg = b"<!DOCTYPE html><html><body><h1>Error: Index not found</h1></body></html>".to_vec();
                (error_msg, "text/html")
            }
        }
    };

    // Write response headers
    let response = if !file_contents.is_empty() && !file_contents.starts_with(b"<!DOCTYPE html><html><body><h1>Error:") {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n",
            final_content_type,
            file_contents.len()
        )
    } else {
        format!(
            "HTTP/1.1 404 NOT FOUND\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
            final_content_type,
            file_contents.len()
        )
    };

    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Failed to write headers: {}", e);
        return;
    }

    // Write file contents
    if let Err(e) = stream.write_all(&file_contents) {
        eprintln!("Failed to write file contents: {}", e);
    }

    let _ = stream.flush();
}
