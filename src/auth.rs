use crate::error::WebauthnError;
use crate::startup::AppState;
use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use tower_sessions::Session;

/*
 * Webauthn RS auth handlers.
 * These files use webauthn to process the data received from each route, and are closely tied to axum
 */

// 1. Import the prelude - this contains everything needed for the server to function.
use webauthn_rs::prelude::*;

#[derive(Deserialize)]
pub struct RegisterStartQuery {
    invite_code: String,
}

// 2. The first step a client (user) will carry out is requesting a credential to be
// registered. We need to provide a challenge for this. The work flow will be:
//
//          ┌───────────────┐     ┌───────────────┐      ┌───────────────┐
//          │ Authenticator │     │    Browser    │      │     Site      │
//          └───────────────┘     └───────────────┘      └───────────────┘
//                  │                     │                      │
//                  │                     │     1. Start Reg     │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│
//                  │                     │                      │
//                  │                     │     2. Challenge     │
//                  │                     │◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┤
//                  │                     │                      │
//                  │  3. Select Token    │                      │
//             ─ ─ ─│◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│                      │
//  4. Verify │     │                     │                      │
//                  │  4. Yield PubKey    │                      │
//            └ ─ ─▶│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶                      │
//                  │                     │                      │
//                  │                     │  5. Send Reg Opts    │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│─ ─ ─
//                  │                     │                      │     │ 5. Verify
//                  │                     │                      │         PubKey
//                  │                     │                      │◀─ ─ ┘
//                  │                     │                      │─ ─ ─
//                  │                     │                      │     │ 6. Persist
//                  │                     │                      │       Credential
//                  │                     │                      │◀─ ─ ┘
//                  │                     │                      │
//                  │                     │                      │
//
// In this step, we are responding to the start reg(istration) request, and providing
// the challenge to the browser.

pub async fn start_register(
    Extension(app_state): Extension<AppState>,
    session: Session,
    Path(username): Path<String>,
    Query(params): Query<RegisterStartQuery>,
) -> Result<impl IntoResponse, WebauthnError> {
    info!("Start register for username: {}", username);

    // Validate invite code first
    let invite_code = app_state
        .database
        .get_invite_code(&params.invite_code)
        .await?;
    let invite_code = match invite_code {
        Some(code) if code.is_active && code.used_at.is_none() => code,
        Some(_) => {
            warn!(
                "Invite code {} is not active or already used",
                params.invite_code
            );
            return Err(WebauthnError::InvalidInviteCode);
        }
        None => {
            warn!("Invite code {} not found", params.invite_code);
            return Err(WebauthnError::InvalidInviteCode);
        }
    };

    // Check if username already exists
    if app_state
        .database
        .get_user_by_username(&username)
        .await?
        .is_some()
    {
        return Err(WebauthnError::UserAlreadyExists);
    }

    // Since a user's username could change at anytime, we need to bind to a unique id.
    // We use uuid's for this purpose, and you should generate these randomly.
    let user_unique_id = Uuid::new_v4();

    // Remove any previous registrations that may have occurred from the session.
    let _ = session.remove_value("reg_state").await;

    // Get existing credentials for this user (should be empty for new users, but good to check)
    let exclude_credentials = app_state
        .database
        .get_user_credentials(user_unique_id)
        .await?
        .iter()
        .map(|sk| sk.cred_id().clone())
        .collect();

    let res = match app_state.webauthn.start_passkey_registration(
        user_unique_id,
        &username,
        &username,
        Some(exclude_credentials),
    ) {
        Ok((ccr, reg_state)) => {
            // Note that due to the session store in use being a server side memory store, this is
            // safe to store the reg_state into the session since it is not client controlled and
            // not open to replay attacks. If this was a cookie store, this would be UNSAFE.
            session
                .insert(
                    "reg_state",
                    (username, user_unique_id, reg_state, invite_code.code),
                )
                .await
                .expect("Failed to insert");
            info!("Registration challenge created successfully!");
            Json(ccr)
        }
        Err(e) => {
            error!("challenge_register -> {:?}", e);
            return Err(WebauthnError::Unknown);
        }
    };
    Ok(res)
}

// 3. The browser has completed its steps and the user has created a public key
// on their device. Now we have the registration options sent to us, and we need
// to verify these and persist them.

pub async fn finish_register(
    Extension(app_state): Extension<AppState>,
    session: Session,
    Json(reg): Json<RegisterPublicKeyCredential>,
) -> Result<impl IntoResponse, WebauthnError> {
    let (username, _user_unique_id, reg_state, invite_code): (
        String,
        Uuid,
        PasskeyRegistration,
        String,
    ) = match session.get("reg_state").await? {
        Some((username, user_unique_id, reg_state, invite_code)) => {
            (username, user_unique_id, reg_state, invite_code)
        }
        None => {
            error!("Failed to get session");
            return Err(WebauthnError::CorruptSession);
        }
    };

    let _ = session.remove_value("reg_state").await;

    let res = match app_state
        .webauthn
        .finish_passkey_registration(&reg, &reg_state)
    {
        Ok(sk) => {
            // Create the user in the database
            match app_state
                .database
                .create_user(&username, Some(&invite_code))
                .await
            {
                Ok(user) => {
                    // Save the credential
                    if let Err(e) = app_state.database.save_credential(user.id, &sk).await {
                        error!("Failed to save credential: {:?}", e);
                        return Err(WebauthnError::DatabaseError);
                    }

                    // Mark the invite code as used
                    if let Err(e) = app_state
                        .database
                        .use_invite_code(&invite_code, user.id)
                        .await
                    {
                        error!("Failed to mark invite code as used: {:?}", e);
                        // Don't fail the registration for this, but log it
                    }

                    info!(
                        "User {} registered successfully with invite code {}",
                        username, invite_code
                    );
                    StatusCode::OK
                }
                Err(e) => {
                    error!("Failed to create user: {:?}", e);
                    return Err(WebauthnError::DatabaseError);
                }
            }
        }
        Err(e) => {
            error!("finish_passkey_registration -> {:?}", e);
            StatusCode::BAD_REQUEST
        }
    };

    Ok(res)
}

// 4. Now that our public key has been registered, we can authenticate a user and verify
// that they are the holder of that security token. The work flow is similar to registration.
//
//          ┌───────────────┐     ┌───────────────┐      ┌───────────────┐
//          │ Authenticator │     │    Browser    │      │     Site      │
//          └───────────────┘     └───────────────┘      └───────────────┘
//                  │                     │                      │
//                  │                     │     1. Start Auth    │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│
//                  │                     │                      │
//                  │                     │     2. Challenge     │
//                  │                     │◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┤
//                  │                     │                      │
//                  │  3. Select Token    │                      │
//             ─ ─ ─│◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│                      │
//  4. Verify │     │                     │                      │
//                  │    4. Yield Sig     │                      │
//            └ ─ ─▶│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶                      │
//                  │                     │    5. Send Auth      │
//                  │                     │        Opts          │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│─ ─ ─
//                  │                     │                      │     │ 5. Verify
//                  │                     │                      │          Sig
//                  │                     │                      │◀─ ─ ┘
//                  │                     │                      │
//                  │                     │                      │
//
// The user indicates the wish to start authentication and we need to provide a challenge.

pub async fn start_authentication(
    Extension(app_state): Extension<AppState>,
    session: Session,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, WebauthnError> {
    info!("Start Authentication for username: {}", username);

    // Remove any previous authentication that may have occurred from the session.
    let _ = session.remove_value("auth_state").await;

    // Look up the user by username
    let user = app_state
        .database
        .get_user_by_username(&username)
        .await?
        .ok_or(WebauthnError::UserNotFound)?;

    // Get the user's credentials
    let allow_credentials = app_state.database.get_user_credentials(user.id).await?;

    if allow_credentials.is_empty() {
        return Err(WebauthnError::UserHasNoCredentials);
    }

    let res = match app_state
        .webauthn
        .start_passkey_authentication(&allow_credentials)
    {
        Ok((rcr, auth_state)) => {
            // Note that due to the session store in use being a server side memory store, this is
            // safe to store the auth_state into the session since it is not client controlled and
            // not open to replay attacks. If this was a cookie store, this would be UNSAFE.
            session
                .insert("auth_state", (user.id, auth_state))
                .await
                .expect("Failed to insert");
            Json(rcr)
        }
        Err(e) => {
            error!("start_passkey_authentication -> {:?}", e);
            return Err(WebauthnError::Unknown);
        }
    };
    Ok(res)
}

// 5. The browser and user have completed their part of the processing. Only in the
// case that the webauthn authenticate call returns Ok, is authentication considered
// a success. If the browser does not complete this call, or *any* error occurs,
// this is an authentication failure.

pub async fn finish_authentication(
    Extension(app_state): Extension<AppState>,
    session: Session,
    Json(auth): Json<PublicKeyCredential>,
) -> Result<impl IntoResponse, WebauthnError> {
    let (user_unique_id, auth_state): (Uuid, PasskeyAuthentication) = session
        .get("auth_state")
        .await?
        .ok_or(WebauthnError::CorruptSession)?;

    let _ = session.remove_value("auth_state").await;

    let res = match app_state
        .webauthn
        .finish_passkey_authentication(&auth, &auth_state)
    {
        Ok(auth_result) => {
            // Get the user's current credentials
            let mut credentials = app_state
                .database
                .get_user_credentials(user_unique_id)
                .await?;

            if credentials.is_empty() {
                return Err(WebauthnError::UserHasNoCredentials);
            }

            // Update the credential counter
            for sk in credentials.iter_mut() {
                sk.update_credential(&auth_result);
                // Save the updated credential back to the database
                if let Err(e) = app_state
                    .database
                    .update_credential(user_unique_id, sk)
                    .await
                {
                    error!("Failed to update credential: {:?}", e);
                    // Don't fail authentication for this, but log it
                }
            }

            info!("Authentication successful for user: {}", user_unique_id);
            StatusCode::OK
        }
        Err(e) => {
            error!("finish_passkey_authentication -> {:?}", e);
            StatusCode::BAD_REQUEST
        }
    };

    Ok(res)
}
