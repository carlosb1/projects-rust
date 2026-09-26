use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    pub success: bool,
}

pub async fn login(
    username: String,
    password: String,
) -> Result<LoginResponse, String> {
    let res = if username == "admin" && password == "admin" {
        LoginResponse{success: true}
    } else {LoginResponse{success: false}};
    Ok(res)
    /*
    let response = Request::post("/api/login")
        .json(&LoginRequest {
            username,
            password,
        })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        return Err(format!(
            "Login failed: {}",
            response.status()
        ));
    }

    response
        .json::<LoginResponse>()
        .await
        .map_err(|e| e.to_string())

     */
}