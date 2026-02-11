mod db;
mod scan;

use actix_cors::Cors;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder, middleware};
use chrono::Utc;
use clap::Parser;
use console::Style;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::OnceCell;
use prometheus::{Registry, IntCounterVec, IntGaugeVec, Opts, TextEncoder, Encoder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use subtle::ConstantTimeEq;
use tokio::sync::Semaphore;
use uuid::Uuid;
use reqwest::Proxy;

use db::DbConnection;

#[derive(Parser, Debug)]
#[command(author = "SPORESEC", version = "1.0")]
struct Args {
    #[arg(long)]
    cli: Option<String>,
    #[arg(long, default_value_t = 5)]
    concurrency: usize,
    #[arg(long, default_value = "../sporesec-dashboard/public/data.json")]
    output: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Claims {
    sub: String,
    exp: usize,
    jti: String,
}

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    semaphore: Arc<Semaphore>,
    link_parallelism: usize,
    db: Arc<tokio::sync::Mutex<DbConnection>>,
}

struct Metrics {
    registry: Registry,
    scans_total: IntCounterVec,
    scan_errors: IntCounterVec,
    active_scans: IntGaugeVec,
}

static METRICS: OnceCell<Metrics> = OnceCell::new();

fn init_metrics() -> Registry {
    let registry = Registry::new();
    let scans_total = IntCounterVec::new(Opts::new("sporesec_scans_total", "Total scans"), &["sector"]).unwrap();
    registry.register(Box::new(scans_total.clone())).unwrap();

    let scan_errors = IntCounterVec::new(Opts::new("sporesec_scan_errors_total", "Total errors"), &["sector"]).unwrap();
    registry.register(Box::new(scan_errors.clone())).unwrap();

    let active_scans = IntGaugeVec::new(Opts::new("sporesec_active_scans", "Active scans"), &["sector"]).unwrap();
    registry.register(Box::new(active_scans.clone())).unwrap();

    let m = Metrics { registry: registry.clone(), scans_total, scan_errors, active_scans };
    let _ = METRICS.set(m);
    registry
}

async fn issue_token(req: HttpRequest, state: web::Data<AppState>, body: web::Json<HashMap<String, String>>) -> impl Responder {
    let admin_key = env::var("ADMIN_API_KEY").unwrap_or_default();
    let header_key = req.headers().get("X-Admin-Key").and_then(|v| v.to_str().ok()).unwrap_or("");
    if header_key != admin_key || admin_key.is_empty() {
        return HttpResponse::Unauthorized().body("Bad admin key");
    }

    let client_id = match body.get("client_id") {
        Some(s) => s.clone(),
        None => return HttpResponse::BadRequest().body("Missing client_id"),
    };

    let secret = env::var("JWT_SECRET").unwrap_or_default();
    if secret.is_empty() {
        return HttpResponse::InternalServerError().body("JWT_SECRET not set");
    }

    let db_guard = state.db.lock().await;
    if let Err(e) = db_guard.ensure_client_in_db(&client_id).await {
        log::error!("Failed to create client: {}", e);
        return HttpResponse::InternalServerError().body("DB error");
    }

    let jti = Uuid::new_v4().to_string();
    let exp = Utc::now().timestamp() as usize + 3600;
    let claims = Claims { sub: client_id.clone(), exp, jti: jti.clone() };

    match encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())) {
        Ok(token) => {
            let db_guard = state.db.lock().await;
            let _ = db_guard.persist_token(&jti, &client_id, exp as i64).await;
            HttpResponse::Ok().json(json!({"token": token, "expires_in": 3600, "jti": jti}))
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Token creation failed: {}", e)),
    }
}

async fn purchase_endpoint(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let auth = match req.headers().get("Authorization").and_then(|v| v.to_str().ok()) {
        Some(s) => s.to_string(),
        None => return HttpResponse::Unauthorized().body("Missing Authorization"),
    };

    if !auth.starts_with("Bearer ") {
        return HttpResponse::Unauthorized().body("Invalid format");
    }

    let token = auth.trim_start_matches("Bearer ").trim();
    let secret = env::var("JWT_SECRET").unwrap_or_default();
    if secret.is_empty() {
        return HttpResponse::InternalServerError().body("JWT_SECRET not set");
    }

    let validation = Validation::new(Algorithm::HS256);
    let token_data = match decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation) {
        Ok(td) => td,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid token"),
    };

    if token_data.claims.exp < Utc::now().timestamp() as usize {
        return HttpResponse::Unauthorized().body("Token expired");
    }

    let db_guard = state.db.lock().await;
    if let Err(e) = db_guard.set_client_paid(&token_data.claims.sub).await {
        log::error!("Failed to mark paid: {}", e);
        return HttpResponse::InternalServerError().body("DB error");
    }

    HttpResponse::Ok().json(json!({"status": "paid", "client_id": token_data.claims.sub, "price": 8.99}))
}

async fn revoke_endpoint(req: HttpRequest, state: web::Data<AppState>, body: web::Json<HashMap<String, String>>) -> impl Responder {
    let admin_key = env::var("ADMIN_API_KEY").unwrap_or_default();
    let header_key = req.headers().get("X-Admin-Key").and_then(|v| v.to_str().ok()).unwrap_or("");
    if header_key != admin_key || admin_key.is_empty() {
        return HttpResponse::Unauthorized().body("Bad admin key");
    }

    let jti = match body.get("jti") {
        Some(j) => j.clone(),
        None => return HttpResponse::BadRequest().body("Missing jti"),
    };

    let db_guard = state.db.lock().await;
    if let Err(e) = db_guard.revoke_token(&jti).await {
        log::error!("Failed to revoke: {}", e);
        return HttpResponse::InternalServerError().body("Revoke failed");
    }

    HttpResponse::Ok().json(json!({"status": "revoked", "jti": jti}))
}

async fn metrics_handler(registry: web::Data<Registry>) -> impl Responder {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = Vec::new();
    let _ = encoder.encode(&metric_families, &mut buffer);
    HttpResponse::Ok().body(String::from_utf8_lossy(&buffer).into_owned())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Security check: Ensure critical keys are set and not using defaults
    let admin_key = env::var("ADMIN_API_KEY").unwrap_or_default();
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_default();
    
    if admin_key.is_empty() || admin_key == "admin-secret" || admin_key == "change-me-at-runtime" {
        log::error!("CRITICAL SECURITY: ADMIN_API_KEY is not set or using a default. Please set a secure key in your environment.");
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Insecure ADMIN_API_KEY"));
    }
    
    if jwt_secret.is_empty() || jwt_secret == "jwt-secret-key" || jwt_secret == "change-me-at-runtime" {
        log::error!("CRITICAL SECURITY: JWT_SECRET is not set or using a default. Please set a secure key in your environment.");
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Insecure JWT_SECRET"));
    }

    let args: Args = Args::parse();
    let title = Style::new().green().bold();

    let proxy = Proxy::all("socks5h://127.0.0.1:9050").expect("Proxy setup failed");
    let client = reqwest::Client::builder()
        .proxy(proxy)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; rv:109.0) Gecko/20100101 Firefox/115.0")
        .connect_timeout(Duration::from_secs(120))
        .timeout(Duration::from_secs(120))
        .pool_max_idle_per_host(0)
        .tcp_keepalive(Duration::from_secs(60))
        .build()
        .expect("Client build failed");

    let semaphore = Arc::new(Semaphore::new(args.concurrency));
    let metrics_registry = init_metrics();

    let db = match DbConnection::init().await {
        Ok(d) => Arc::new(tokio::sync::Mutex::new(d)),
        Err(e) => {
            log::error!("DB init failed: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e)));
        }
    };

    if let Some(query) = args.cli {
        println!("{}", title.apply_to("--- SPORESEC CLI ---"));
        println!("[*] Query: {}", query);
        let sectors: Vec<(String, String)> = vec![
            (format!("https://ahmia.fi/search/?q={}", query), "Ahmia-Search".to_string()),
            ("https://tor.taxi/".to_string(), "TorTaxi".to_string()),
            ("https://darknetlive.com/markets/".to_string(), "DarknetLive".to_string()),
        ];
        let mut all_results = Vec::new();
        for (url, name) in sectors {
            let res = scan::scan_sector_improved(&client, &url, &name, semaphore.clone(), 6).await;
            all_results.extend(res);
        }
        println!("[+] Found {} results", all_results.len());
        if let Ok(s) = serde_json::to_string_pretty(&all_results) {
            let _ = std::fs::write(&args.output, s);
            println!("Saved to {}", &args.output);
        }
        return Ok(());
    }

    println!("{}", title.apply_to("--- SPORESEC API SERVER ---"));
    let frontend = env::var("FRONTEND_DOMAIN").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let state = AppState { client, semaphore, link_parallelism: 6, db };

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&frontend)
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec!["Authorization", "Content-Type", "X-Admin-Key", "X-Spore-Signature"])
            .max_age(3600);

        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(metrics_registry.clone()))
            .route("/v1/token", web::post().to(issue_token))
            .route("/v1/purchase", web::post().to(purchase_endpoint))
            .route("/v1/revoke", web::post().to(revoke_endpoint))
            .route("/metrics", web::get().to(metrics_handler))
    })
    .bind("0.0.0.0:8080")?
    .run();

    let handle = server.handle();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        log::info!("Shutting down...");
        handle.stop(true).await;
    });

    server.await
}