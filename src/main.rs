use axum::{
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, header, StatusCode},
    response::{IntoResponse, Response},
    routing::{post, get},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use reqwest::Client as HttpClient;
use serde::{Deserialize};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Deserialize)]
struct PdfRequest {
    html: String,
    #[serde(default)]
    landscape: bool,
}

struct AppState {
    http: HttpClient,
    // 1. Concurrency Control: Limits parallel browser tabs
    semaphore: Semaphore,
}

#[tokio::main]
async fn main() {
    // Limit to 4 parallel renders to prevent server crash
    let state = Arc::new(AppState {
        http: HttpClient::new(),
        semaphore: Semaphore::new(4), 
    });

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/generate", post(generate_pdf))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Production Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn generate_pdf(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PdfRequest>,
) -> Response {
    
    // 1. Acquire Permit (Wait here if busy)
    let _permit = match state.semaphore.acquire().await {
        Ok(p) => p,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Semaphore failed").into_response(),
    };

    // 2. Hard Timeout (10 Seconds)
    // If generation hangs, we kill it to free up the slot
    let result = timeout(Duration::from_secs(10), process_pdf(&state.http, payload)).await;

    match result {
        Ok(inner_result) => match inner_result {
            Ok(bytes) => {
                let mut headers = HeaderMap::new();
                headers.insert(header::CONTENT_TYPE, "application/pdf".parse().unwrap());
                headers.insert(header::CONTENT_DISPOSITION, "attachment; filename=\"output.pdf\"".parse().unwrap());
                (headers, bytes).into_response()
            },
            Err(e) => {
                eprintln!("PDF Gen Error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
            }
        },
        Err(_) => {
            eprintln!("Request Timed Out!");
            (StatusCode::REQUEST_TIMEOUT, "Request timed out").into_response()
        }
    }
}

async fn process_pdf(http: &HttpClient, payload: PdfRequest) -> Result<Vec<u8>, String> {
    
    // A. Create Tab via HTTP
    let new_tab_url = "http://127.0.0.1:9222/json/new";
    let tab_info: Value = http.put(new_tab_url).send().await
        .map_err(|e| format!("Chrome connection failed: {}", e))?
        .json().await
        .map_err(|e| format!("Invalid Chrome response: {}", e))?;

    let ws_url = tab_info["webSocketDebuggerUrl"].as_str().ok_or("No WS URL found")?;
    let tab_id = tab_info["id"].as_str().ok_or("No Tab ID found")?;

    // Cleanup URL (Called on success or error)
    let close_url = format!("http://127.0.0.1:9222/json/close/{}", tab_id);
    
    // B. Connect WS
    let (ws_stream, _) = match connect_async(ws_url).await {
        Ok(v) => v,
        Err(e) => {
            let _ = http.get(&close_url).send().await; 
            return Err(format!("WS Connect failed: {}", e));
        }
    };
    let (mut write, mut read) = ws_stream.split();

    let mut command_id = 0;

    // --- C. Set Content ---
    command_id += 1;
    let set_content_cmd = json!({
        "id": command_id,
        "method": "Page.setDocumentContent",
        "params": {
            "frameId": tab_id,
            "html": payload.html
        }
    });

    if let Err(e) = write.send(Message::Text(set_content_cmd.to_string())).await {
        let _ = http.get(&close_url).send().await;
        return Err(e.to_string());
    }

    // Wait for ACK
    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            if text.contains(&format!("\"id\":{}", command_id)) { break; }
        }
    }

    // --- D. Print to PDF (No Delay) ---
    command_id += 1;
    let print_cmd = json!({
        "id": command_id,
        "method": "Page.printToPDF",
        "params": {
            "landscape": payload.landscape,
            "printBackground": true,
            "paperWidth": 8.27,
            "paperHeight": 11.7,
            "marginTop": 0.0,
            "marginBottom": 0.0,
            "marginLeft": 0.0,
            "marginRight": 0.0
        }
    });

    if let Err(e) = write.send(Message::Text(print_cmd.to_string())).await {
        let _ = http.get(&close_url).send().await;
        return Err(e.to_string());
    }

    let mut pdf_base64 = String::new();
    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            if text.contains(&format!("\"id\":{}", command_id)) {
                let v: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                if let Some(data) = v["result"]["data"].as_str() {
                    pdf_base64 = data.to_string();
                } else {
                     let _ = http.get(&close_url).send().await;
                     return Err("Chrome returned no data".to_string());
                }
                break;
            }
        }
    }

    // --- E. Cleanup & Return ---
    let _ = http.get(&close_url).send().await;

    if pdf_base64.is_empty() {
        return Err("Empty PDF data".to_string());
    }

    base64::decode(&pdf_base64).map_err(|e| e.to_string())
}