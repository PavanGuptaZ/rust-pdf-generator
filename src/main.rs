use axum::{
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, header},
    response::IntoResponse,
    routing::{post, get},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use reqwest::Client as HttpClient;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Deserialize)]
struct PdfRequest {
    html: String,
    #[serde(default)]
    landscape: bool,
}

struct AppState {
    http: HttpClient,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        http: HttpClient::new(),
    });

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/generate", post(generate_pdf))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 High-Perf Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn generate_pdf(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PdfRequest>,
) -> Result<impl IntoResponse, String> {
    let new_tab_url = "http://127.0.0.1:9222/json/new";
    let tab_info: Value = state.http.put(new_tab_url).send().await
        .map_err(|e| format!("Chrome not reachable: {}", e))?
        .json().await
        .map_err(|e| format!("Failed to parse tab info: {}", e))?;

    let ws_url = tab_info["webSocketDebuggerUrl"].as_str()
        .ok_or("No WebSocket URL found")?;
    let tab_id = tab_info["id"].as_str()
        .ok_or("No Tab ID found")?;

    let (ws_stream, _) = connect_async(ws_url).await
        .map_err(|e| format!("WS Connect failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    let mut command_id = 0;
    
    command_id += 1;
    let set_content_cmd = json!({
        "id": command_id,
        "method": "Page.setDocumentContent",
        "params": {
            "frameId": tab_id,
            "html": payload.html
        }
    });
    write.send(Message::Text(set_content_cmd.to_string())).await.map_err(|e| e.to_string())?;

    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            if text.contains(&format!("\"id\":{}", command_id)) { break; }
        }
    }

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
    write.send(Message::Text(print_cmd.to_string())).await.map_err(|e| e.to_string())?;

    let mut pdf_base64 = String::new();
    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            if text.contains(&format!("\"id\":{}", command_id)) {
                let v: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                if let Some(data) = v["result"]["data"].as_str() {
                    pdf_base64 = data.to_string();
                }
                break;
            }
        }
    }

    let close_url = format!("http://127.0.0.1:9222/json/close/{}", tab_id);
    let _ = state.http.get(&close_url).send().await;

    if pdf_base64.is_empty() {
        return Err("PDF generation failed (empty response)".to_string());
    }

    let pdf_bytes = base64::decode(&pdf_base64).map_err(|e| e.to_string())?;

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/pdf".parse().unwrap());
    headers.insert(header::CONTENT_DISPOSITION, "attachment; filename=\"output.pdf\"".parse().unwrap());

    Ok((headers, pdf_bytes))
}