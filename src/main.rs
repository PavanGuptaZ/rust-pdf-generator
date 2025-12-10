use axum::{
    extract::{DefaultBodyLimit},
    http::{HeaderMap, header},
    response::{IntoResponse},
    routing::{post, get},
    Json, Router,
};
use serde::Deserialize;
use std::process::Command;
use std::fs;
use uuid::Uuid;

#[derive(Deserialize)]
struct PdfRequest {
    html: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/generate", post(generate_pdf))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn generate_pdf(Json(payload): Json<PdfRequest>) -> impl IntoResponse {
    let file_id = Uuid::new_v4();
    let input_path = format!("/tmp/{}.html", file_id);
    let output_path = format!("/tmp/{}.pdf", file_id);

    // 1. Write HTML to file
    if let Err(_) = fs::write(&input_path, &payload.html) {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to write HTML").into_response();
    }

    // 2. Run Chromium CLI
    // --headless=new is the modern headless mode
    let status = Command::new("chromium")
        .args(&[
            "--headless=new",
            "--disable-gpu",
            "--no-pdf-header-footer",
            "--print-to-pdf-no-header",
            &format!("--print-to-pdf={}", output_path),
            &input_path
        ])
        .status();

    // 3. Check if command succeeded
    match status {
        Ok(s) if s.success() => {
            // Read the generated PDF
            match fs::read(&output_path) {
                Ok(pdf_bytes) => {
                    // Cleanup
                    let _ = fs::remove_file(input_path);
                    let _ = fs::remove_file(output_path);

                    let mut headers = HeaderMap::new();
                    headers.insert(header::CONTENT_TYPE, "application/pdf".parse().unwrap());
                    headers.insert(header::CONTENT_DISPOSITION, "attachment; filename=\"output.pdf\"".parse().unwrap());
                    
                    (headers, pdf_bytes).into_response()
                },
                Err(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to read PDF").into_response()
            }
        },
        _ => {
            // Cleanup
            let _ = fs::remove_file(input_path);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Chromium failed to generate PDF").into_response()
        }
    }
}