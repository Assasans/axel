pub mod format;
pub mod totp;
pub mod validation;

use std::sync::Arc;
use std::time::{Instant};

use crate::sql_console::format::format_cell;
use crate::sql_console::totp::verify_totp;
use crate::sql_console::validation::validate_sql;
use crate::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info, warn};

#[derive(Debug, Deserialize)]
pub struct AuthTokenRequest {
  pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthOtpRequest {
  #[serde(rename = "userId")]
  pub user_id: i64,
  pub otp: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
  pub success: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub token: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user: Option<UserInfo>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
  pub id: i64,
  pub name: String,
  pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
  pub sql: String,
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
  pub success: bool,
  pub results: Vec<StatementResult>,
}

#[derive(Debug, Serialize)]
pub struct StatementResult {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sql: Option<String>,
  pub success: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub result: Option<QueryResult>,
  #[serde(rename = "rowsAffected", skip_serializing_if = "Option::is_none")]
  pub rows_affected: Option<u64>,
  #[serde(rename = "executionTime", skip_serializing_if = "Option::is_none")]
  pub execution_time: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QueryResult {
  pub columns: Vec<QueryColumn>,
  pub rows: Vec<Vec<Value>>,
}

#[derive(Debug, Serialize)]
pub struct QueryColumn {
  pub name: String,
  #[serde(rename = "type")]
  pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
  pub error: String,
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
  headers
    .get("Authorization")
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.strip_prefix("Bearer "))
    .map(|s| s.to_string())
}

async fn validate_auth_token(state: &AppState, token: &str) -> Option<UserInfo> {
  let pool = &state.pool;
  let client = match pool.get().await {
    Ok(c) => c,
    Err(e) => {
      error!("Failed to get database connection for auth: {:?}", e);
      return None;
    }
  };

  // Look up token in user_devices and join with users
  let query = /* language=postgresql */ r#"
    select u.id, u.username
    from user_devices ud
    join users u on u.id = ud.user_id
    where ud.token = $1
  "#;

  match client.query_opt(query, &[&token]).await {
    Ok(Some(row)) => {
      let user_id: i64 = row.get("id");
      let name: String = row.get("username");
      Some(UserInfo {
        id: user_id,
        name,
        role: "admin".to_string(), // Default role, could be fetched from DB
      })
    }
    Ok(None) => {
      warn!("No user found for token");
      None
    }
    Err(error) => {
      error!("Database error during token validation: {:?}", error);
      None
    }
  }
}

async fn validate_otp_auth(state: &AppState, user_id: i64, otp: &str) -> Option<UserInfo> {
  let pool = &state.pool;
  let client = match pool.get().await {
    Ok(c) => c,
    Err(error) => {
      error!("Failed to get database connection for OTP auth: {:?}", error);
      return None;
    }
  };

  // Get user info
  let user_query = /* language=postgresql */ "select id, username from users where id = $1";
  let user_row = match client.query_opt(user_query, &[&user_id]).await {
    Ok(Some(row)) => row,
    Ok(None) => {
      warn!("no user found with id {}", user_id);
      return None;
    }
    Err(error) => {
      error!("database error fetching user: {:?}", error);
      return None;
    }
  };

  let name: String = user_row.get("username");

  // Get all devices for this user and check TOTP against each
  let devices_query = /* language=postgresql */ "select token from user_devices where user_id = $1";
  let devices = match client.query(devices_query, &[&user_id]).await {
    Ok(rows) => rows,
    Err(e) => {
      error!("database error fetching user devices: {:?}", e);
      return None;
    }
  };

  if devices.is_empty() {
    warn!("no devices found for user {}", user_id);
    return None;
  }

  // Check OTP against each device's token as the TOTP secret
  for device_row in devices {
    let token: String = device_row.get("token");
    let secret = token.as_bytes();

    if verify_totp(secret, otp, 60) {
      return Some(UserInfo {
        id: user_id,
        name,
        role: "admin".to_string(),
      });
    }
  }

  warn!("OTP verification failed for user {}", user_id);
  None
}

async fn auth_token(State(state): State<Arc<AppState>>, Json(req): Json<AuthTokenRequest>) -> impl IntoResponse {
  info!("SQL console token auth attempt");

  match validate_auth_token(&state, &req.token).await {
    Some(user) => {
      info!("SQL console auth successful for user: {} (id={})", user.name, user.id);
      (
        StatusCode::OK,
        Json(AuthResponse {
          success: true,
          token: Some(req.token),
          user: Some(user),
          error: None,
        }),
      )
    }
    None => {
      warn!("SQL console auth failed: invalid token");
      (
        StatusCode::UNAUTHORIZED,
        Json(AuthResponse {
          success: false,
          token: None,
          user: None,
          error: Some("invalid token".to_string()),
        }),
      )
    }
  }
}

async fn auth_otp(State(state): State<Arc<AppState>>, Json(req): Json<AuthOtpRequest>) -> impl IntoResponse {
  info!("SQL console OTP auth attempt for user_id={}", req.user_id);

  match validate_otp_auth(&state, req.user_id, &req.otp).await {
    Some(user) => {
      // Generate a session token (use one of the user's device tokens)
      let pool = &state.pool;
      let token = match pool.get().await {
        Ok(client) => {
          match client
            .query_opt(
              /* language=postgresql */ "select token from user_devices where user_id = $1 limit 1",
              &[&req.user_id],
            )
            .await
          {
            Ok(Some(row)) => row.get::<_, String>("token"),
            _ => {
              return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                  success: false,
                  token: None,
                  user: None,
                  error: Some("failed to generate session token".to_string()),
                }),
              );
            }
          }
        }
        Err(error) => {
          error!("database error: {:?}", error);
          return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthResponse {
              success: false,
              token: None,
              user: None,
              error: Some("database error".to_string()),
            }),
          );
        }
      };

      info!(
        "SQL console OTP auth successful for user: {} (id={})",
        user.name, user.id
      );
      (
        StatusCode::OK,
        Json(AuthResponse {
          success: true,
          token: Some(token),
          user: Some(user),
          error: None,
        }),
      )
    }
    None => {
      warn!("SQL console OTP auth failed for user_id={}", req.user_id);
      (
        StatusCode::UNAUTHORIZED,
        Json(AuthResponse {
          success: false,
          token: None,
          user: None,
          error: Some("invalid OTP".to_string()),
        }),
      )
    }
  }
}

async fn query(
  State(state): State<Arc<AppState>>,
  headers: HeaderMap,
  Json(req): Json<QueryRequest>,
) -> impl IntoResponse {
  // Validate authorization
  let token = match extract_bearer_token(&headers) {
    Some(t) => t,
    None => {
      return (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": "missing authorization header" })),
      );
    }
  };

  let user = match validate_auth_token(&state, &token).await {
    Some(u) => u,
    None => {
      return (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": "invalid or expired token" })),
      );
    }
  };

  let sql = req.sql.trim();
  if sql.is_empty() {
    return (
      StatusCode::OK,
      Json(serde_json::json!({
        "success": true,
        "results": []
      })),
    );
  }

  // Validate SQL
  if let Err(reason) = validate_sql(sql) {
    warn!("SQL query blocked: {}", reason);
    return (
      StatusCode::OK,
      Json(serde_json::json!({
        "success": false,
        "results": [{
          "sql": sql,
          "success": false,
          "error": reason
        }]
      })),
    );
  }

  // Parse statements
  let parsed = match pg_query::parse(sql) {
    Ok(p) => p,
    Err(e) => {
      return (
        StatusCode::OK,
        Json(serde_json::json!({
          "success": false,
          "results": [{
            "success": false,
            "error": format!("parse error: {}", e)
          }]
        })),
      );
    }
  };

  // Execute query
  let pool = &state.sql_console_pool; // changed (was state.pool)
  let mut client = match pool.get().await {
    Ok(c) => c,
    Err(error) => {
      error!("failed to get database connection: {:?}", error);
      return (
        StatusCode::OK,
        Json(serde_json::json!({
          "success": false,
          "results": [{
            "success": false,
            "error": format!("database connection error: {}", error)
          }]
        })),
      );
    }
  };

  let transaction = match client.transaction().await {
    Ok(t) => t,
    Err(error) => {
      error!("failed to start transaction: {:?}", error);
      return (
        StatusCode::OK,
        Json(serde_json::json!({
          "success": false,
          "results": [{
            "success": false,
            "error": format!("transaction error: {}", error)
          }]
        })),
      );
    }
  };

  // Set app.user_id for RLS policies using the authenticated user's ID
  let set_user_sql = format!("SET LOCAL app.user_id = '{}'", user.id);
  if let Err(error) = transaction.execute(&set_user_sql, &[]).await {
    error!("failed to set app.user_id: {:?}", error);
  }

  let mut results: Vec<StatementResult> = Vec::new();
  let mut all_success = true;

  for stmt in parsed.protobuf.stmts.iter() {
    let stmt_sql = match pg_query::deparse(&pg_query::protobuf::ParseResult {
      version: parsed.protobuf.version,
      stmts: vec![stmt.clone()],
    }) {
      Ok(s) => s,
      Err(error) => {
        results.push(StatementResult {
          sql: None,
          success: false,
          result: None,
          rows_affected: None,
          execution_time: None,
          error: Some(format!("Deparse error: {}", error)),
        });
        all_success = false;
        continue;
      }
    };

    debug!("executing SQL statement: {}", stmt_sql.trim());
    let start = Instant::now();

    match transaction.query(&stmt_sql, &[]).await {
      Ok(rows) => {
        let execution_time = start.elapsed().as_secs_f64() * 1000.0;

        if rows.is_empty() {
          results.push(StatementResult {
            sql: Some(stmt_sql.trim().to_string()),
            success: true,
            result: Some(QueryResult {
              columns: vec![],
              rows: vec![],
            }),
            rows_affected: Some(0),
            execution_time: Some(execution_time),
            error: None,
          });
        } else {
          let columns: Vec<QueryColumn> = rows[0]
            .columns()
            .iter()
            .map(|c| QueryColumn {
              name: c.name().to_string(),
              kind: c.type_().name().to_string(),
            })
            .collect();

          let row_data: Vec<Vec<Value>> = rows
            .iter()
            .take(1000) // Limit to 1000 rows
            .map(|row| (0..row.len()).map(|i| format_cell(row, i)).collect())
            .collect();

          results.push(StatementResult {
            sql: Some(stmt_sql.trim().to_string()),
            success: true,
            result: Some(QueryResult {
              columns,
              rows: row_data,
            }),
            rows_affected: Some(rows.len() as u64),
            execution_time: Some(execution_time),
            error: None,
          });
        }
      }
      Err(e) => {
        error!("SQL execution error: {:?}", e);
        results.push(StatementResult {
          sql: Some(stmt_sql.trim().to_string()),
          success: false,
          result: None,
          rows_affected: None,
          execution_time: Some(start.elapsed().as_secs_f64() * 1000.0),
          error: Some(e.to_string()),
        });
        all_success = false;
      }
    }
  }

  // Commit transaction
  if let Err(error) = transaction.commit().await {
    error!("failed to commit transaction: {:?}", error);
    return (
      StatusCode::OK,
      Json(serde_json::json!({
        "success": false,
        "results": [{
          "success": false,
          "error": format!("commit error: {}", error)
        }]
      })),
    );
  }

  info!(
    "SQL query executed by user {} (id={}): {} statements, success={}",
    user.name,
    user.id,
    results.len(),
    all_success
  );

  (
    StatusCode::OK,
    Json(serde_json::json!({
      "success": all_success,
      "results": results
    })),
  )
}

pub fn router() -> Router<Arc<AppState>> {
  Router::new()
    .route("/auth/token", post(auth_token))
    .route("/auth/otp", post(auth_otp))
    .route("/query", post(query))
    .layer(CorsLayer::permissive())
}
