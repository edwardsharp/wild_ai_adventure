use std::fs;
use utoipa::OpenApi;
use utoipa::{openapi, Modify};

// This would normally import your actual API types
// For now, we'll define some example types that match your WebAuthn API

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
pub struct RegisterStartRequest {
    pub username: String,
    pub display_name: String,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
pub struct RegisterStartResponse {
    pub challenge: String,
    pub rp: RelyingParty,
    pub user: User,
    pub pub_key_cred_params: Vec<PubKeyCredParam>,
    pub authenticator_selection: AuthenticatorSelection,
    pub timeout: u32,
    pub attestation: String,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
pub struct RelyingParty {
    pub id: String,
    pub name: String,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub display_name: String,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
pub struct PubKeyCredParam {
    pub alg: i32,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
pub struct AuthenticatorSelection {
    pub authenticator_attachment: Option<String>,
    pub require_resident_key: bool,
    pub resident_key: String,
    pub user_verification: String,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
pub struct RegisterFinishRequest {
    pub user: RegisterStartRequest,
    pub credential: PublicKeyCredential,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
pub struct PublicKeyCredential {
    pub id: String,
    #[serde(rename = "rawId")]
    pub raw_id: String,
    pub response: AuthenticatorAttestationResponse,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
pub struct AuthenticatorAttestationResponse {
    #[serde(rename = "attestationObject")]
    pub attestation_object: String,
    #[serde(rename = "clientDataJSON")]
    pub client_data_json: String,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
pub struct LoginStartRequest {
    pub username: String,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
pub struct LoginStartResponse {
    pub challenge: String,
    pub timeout: u32,
    pub rp_id: String,
    pub allow_credentials: Vec<AllowCredential>,
    pub user_verification: String,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
pub struct AllowCredential {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub transports: Vec<String>,
}

#[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        register_start,
        register_finish,
        login_start,
        login_finish,
        logout,
        health_check
    ),
    components(
        schemas(
            RegisterStartRequest,
            RegisterStartResponse,
            RegisterFinishRequest,
            LoginStartRequest,
            LoginStartResponse,
            PublicKeyCredential,
            AuthenticatorAttestationResponse,
            RelyingParty,
            User,
            PubKeyCredParam,
            AuthenticatorSelection,
            AllowCredential,
            ErrorResponse
        )
    ),
    tags(
        (name = "auth", description = "WebAuthn authentication endpoints"),
        (name = "health", description = "Health check endpoints")
    ),
    info(
        title = "WebAuthn API",
        description = "A WebAuthn-based authentication API",
        version = "1.0.0"
    )
)]
struct ApiDoc;

// Mock path functions for OpenAPI generation
#[utoipa::path(
    post,
    path = "/register_start",
    tag = "auth",
    request_body = RegisterStartRequest,
    responses(
        (status = 200, description = "Registration challenge created", body = RegisterStartResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
async fn register_start() {}

#[utoipa::path(
    post,
    path = "/register_finish",
    tag = "auth",
    request_body = RegisterFinishRequest,
    responses(
        (status = 200, description = "Registration completed successfully"),
        (status = 400, description = "Invalid credential", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
async fn register_finish() {}

#[utoipa::path(
    post,
    path = "/login_start",
    tag = "auth",
    request_body = LoginStartRequest,
    responses(
        (status = 200, description = "Login challenge created", body = LoginStartResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
async fn login_start() {}

#[utoipa::path(
    post,
    path = "/login_finish",
    tag = "auth",
    request_body = PublicKeyCredential,
    responses(
        (status = 200, description = "Login successful"),
        (status = 400, description = "Invalid credential", body = ErrorResponse),
        (status = 401, description = "Authentication failed", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
async fn login_finish() {}

#[utoipa::path(
    post,
    path = "/logout",
    tag = "auth",
    responses(
        (status = 200, description = "Logout successful"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
async fn logout() {}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy"),
        (status = 500, description = "Service is unhealthy")
    )
)]
async fn health_check() {}

fn main() {
    let openapi = ApiDoc::openapi();

    // Write OpenAPI spec as JSON
    let json_spec = serde_json::to_string_pretty(&openapi)
        .expect("Failed to serialize OpenAPI spec");

    fs::write("../generated/openapi.json", json_spec)
        .expect("Failed to write OpenAPI JSON file");

    // Write OpenAPI spec as YAML
    let yaml_spec = serde_yaml::to_string(&openapi)
        .expect("Failed to serialize OpenAPI spec to YAML");

    fs::write("../generated/openapi.yaml", yaml_spec)
        .expect("Failed to write OpenAPI YAML file");

    println!("OpenAPI specification generated successfully!");
    println!("- JSON: generated/openapi.json");
    println!("- YAML: generated/openapi.yaml");
}
