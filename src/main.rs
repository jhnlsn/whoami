use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpListener;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

// Response structure for JSON endpoint
#[derive(Serialize, Deserialize)]
struct WhoAmIResponse {
    ip: String,
    user_agent: String,
    headers: HashMap<String, String>,
}

// HTML template
const HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WhoAmI - Your Connection Info</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
            padding: 20px;
        }
        .container {
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            max-width: 800px;
            width: 100%;
            padding: 40px;
        }
        h1 {
            color: #333;
            margin-bottom: 10px;
            font-size: 2.5em;
        }
        .subtitle {
            color: #666;
            margin-bottom: 30px;
            font-size: 1.1em;
        }
        .info-card {
            background: #f8f9fa;
            border-left: 4px solid #667eea;
            padding: 20px;
            margin-bottom: 20px;
            border-radius: 4px;
        }
        .info-label {
            color: #667eea;
            font-weight: bold;
            font-size: 0.9em;
            text-transform: uppercase;
            letter-spacing: 1px;
            margin-bottom: 8px;
        }
        .info-value {
            color: #333;
            font-size: 1.2em;
            word-break: break-all;
            font-family: 'Courier New', monospace;
        }
        .headers-section {
            margin-top: 30px;
        }
        .headers-title {
            color: #333;
            font-size: 1.5em;
            margin-bottom: 15px;
            padding-bottom: 10px;
            border-bottom: 2px solid #667eea;
        }
        .header-item {
            background: white;
            padding: 12px;
            margin-bottom: 8px;
            border-radius: 4px;
            border: 1px solid #e0e0e0;
            font-family: 'Courier New', monospace;
            font-size: 0.9em;
        }
        .header-name {
            color: #667eea;
            font-weight: bold;
        }
        .header-value {
            color: #333;
            margin-left: 10px;
        }
        .footer {
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #e0e0e0;
            text-align: center;
            color: #999;
            font-size: 0.9em;
        }
        .api-links {
            display: flex;
            gap: 10px;
            margin-top: 20px;
            flex-wrap: wrap;
        }
        .api-link {
            background: #667eea;
            color: white;
            padding: 10px 20px;
            border-radius: 6px;
            text-decoration: none;
            font-weight: 500;
            transition: background 0.3s;
        }
        .api-link:hover {
            background: #764ba2;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔍 WhoAmI</h1>
        <p class="subtitle">Your connection information</p>

        <div class="info-card">
            <div class="info-label">IP Address</div>
            <div class="info-value">{IP_ADDRESS}</div>
        </div>

        <div class="info-card">
            <div class="info-label">User Agent</div>
            <div class="info-value">{USER_AGENT}</div>
        </div>

        <div class="headers-section">
            <h2 class="headers-title">Request Headers</h2>
            {HEADERS}
        </div>

        <div class="api-links">
            <a href="/json" class="api-link">JSON API</a>
            <a href="/text" class="api-link">Plain Text</a>
        </div>

        <div class="footer">
            Powered by Rust + Hyper | Ultra-lightweight HTTP service
        </div>
    </div>
</body>
</html>"#;

// Extract client IP from request headers or peer address
fn extract_client_ip(req: &Request<Incoming>, peer_addr: SocketAddr) -> String {
    // Check X-Forwarded-For first (common with proxies/load balancers)
    if let Some(xff) = req.headers().get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            if let Some(first_ip) = xff_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    // Check X-Real-IP (nginx)
    if let Some(xri) = req.headers().get("x-real-ip") {
        if let Ok(ip) = xri.to_str() {
            return ip.to_string();
        }
    }

    // Fallback to peer address
    peer_addr.ip().to_string()
}

// Extract user agent from request headers
fn extract_user_agent(req: &Request<Incoming>) -> String {
    req.headers()
        .get("user-agent")
        .and_then(|ua| ua.to_str().ok())
        .unwrap_or("Unknown")
        .to_string()
}

// Collect all headers into a HashMap
fn collect_headers(req: &Request<Incoming>) -> HashMap<String, String> {
    req.headers()
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|v| (name.as_str().to_string(), v.to_string()))
        })
        .collect()
}

// Serve JSON response
async fn serve_json(
    req: Request<Incoming>,
    peer_addr: SocketAddr,
) -> Result<Response<BoxBody>, Infallible> {
    let ip = extract_client_ip(&req, peer_addr);
    let user_agent = extract_user_agent(&req);
    let headers = collect_headers(&req);

    let response = WhoAmIResponse {
        ip,
        user_agent,
        headers,
    };

    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string());

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .header("access-control-allow-origin", "*")
        .body(
            Full::new(Bytes::from(json))
                .map_err(|e| match e {})
                .boxed(),
        )
        .unwrap();

    Ok(response)
}

// Serve plain text response
async fn serve_text(
    req: Request<Incoming>,
    peer_addr: SocketAddr,
) -> Result<Response<BoxBody>, Infallible> {
    let ip = extract_client_ip(&req, peer_addr);
    let user_agent = extract_user_agent(&req);
    let headers = collect_headers(&req);

    let mut text = format!("Client IP: {}\n", ip);
    text.push_str(&format!("User-Agent: {}\n\n", user_agent));
    text.push_str("Request Headers:\n");

    for (name, value) in headers.iter() {
        text.push_str(&format!("  {}: {}\n", name, value));
    }

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/plain; charset=utf-8")
        .body(
            Full::new(Bytes::from(text))
                .map_err(|e| match e {})
                .boxed(),
        )
        .unwrap();

    Ok(response)
}

// Serve HTML response
async fn serve_html(
    req: Request<Incoming>,
    peer_addr: SocketAddr,
) -> Result<Response<BoxBody>, Infallible> {
    let ip = extract_client_ip(&req, peer_addr);
    let user_agent = extract_user_agent(&req);
    let headers = collect_headers(&req);

    // Build headers HTML
    let mut headers_html = String::new();
    for (name, value) in headers.iter() {
        headers_html.push_str(&format!(
            r#"<div class="header-item"><span class="header-name">{}</span><span class="header-value">{}</span></div>"#,
            html_escape(name),
            html_escape(value)
        ));
    }

    let html = HTML_TEMPLATE
        .replace("{IP_ADDRESS}", &html_escape(&ip))
        .replace("{USER_AGENT}", &html_escape(&user_agent))
        .replace("{HEADERS}", &headers_html);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html; charset=utf-8")
        .body(
            Full::new(Bytes::from(html))
                .map_err(|e| match e {})
                .boxed(),
        )
        .unwrap();

    Ok(response)
}

// Serve health check
async fn serve_health() -> Result<Response<BoxBody>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/plain")
        .body(
            Full::new(Bytes::from("OK"))
                .map_err(|e| match e {})
                .boxed(),
        )
        .unwrap();

    Ok(response)
}

// Serve 404 response
async fn serve_404() -> Result<Response<BoxBody>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("content-type", "text/plain")
        .body(
            Full::new(Bytes::from("404 Not Found"))
                .map_err(|e| match e {})
                .boxed(),
        )
        .unwrap();

    Ok(response)
}

// Simple HTML escaping
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// Main request handler with routing
async fn handle_request(
    req: Request<Incoming>,
    peer_addr: SocketAddr,
) -> Result<Response<BoxBody>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => serve_html(req, peer_addr).await,
        (&Method::GET, "/json") => serve_json(req, peer_addr).await,
        (&Method::GET, "/api") => serve_json(req, peer_addr).await,
        (&Method::GET, "/text") => serve_text(req, peer_addr).await,
        (&Method::GET, "/health") => serve_health().await,
        _ => serve_404().await,
    }
}

// Get port from environment variable or use default
fn get_port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = get_port();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = TcpListener::bind(addr).await?;
    println!("🚀 WhoAmI server listening on http://{}", addr);
    println!("📍 Endpoints:");
    println!("   GET /       - HTML page");
    println!("   GET /json   - JSON API");
    println!("   GET /api    - JSON API (alias)");
    println!("   GET /text   - Plain text");
    println!("   GET /health - Health check");

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| handle_request(req, peer_addr)),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
