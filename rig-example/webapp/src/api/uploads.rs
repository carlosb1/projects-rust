use gloo_net::http::Request;

pub async fn upload_zip(
    filename: String,
    data: Vec<u8>,
) -> Result<(), String> {
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

    Ok(())
}

use crate::models::upload::UploadReport;

pub async fn get_upload_report(
    upload_id: &str,
) -> Result<UploadReport, String> {
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
        .json::<UploadReport>()
        .await
        .map_err(|e| e.to_string())
}