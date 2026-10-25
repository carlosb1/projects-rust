use gloo_net::http::Request;
use uuid::Uuid;

pub async fn upload_zip(
    filename: String,
    data: Vec<u8>,
) -> Result<Uuid, String> {
    let response = Request::post("/api/uploads")
        .header("Content-Type", "application/zip")
        .header("X-Filename", &filename)
        .body(data)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        return Err(format!(
            "Upload failed: {}",
            response.status()
        ));
    }

    let uuid = response
        .json::<Uuid>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(uuid)
}

use crate::models::upload::Report;

pub async fn get_upload_report(
    upload_id: &str,
) -> Result<Report, String> {
    let url = format!("/api/uploads/{upload_id}/report");

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        return Err(format!(
            "Failed to load report: {}",
            response.status()
        ));
    }

    response
        .json::<Report>()
        .await
        .map_err(|e| e.to_string())
}