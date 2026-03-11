//! # webThing
//!
//! Proof of concept IoT REST HTTP JSON server using rust actix and SQLite through sqlx.
//!

use actix_files::Files;
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::{
    App, Error, HttpResponse, HttpServer, Result, cookie,
    error::{ErrorInternalServerError, ErrorUnauthorized},
    middleware::Logger,
    web,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

/// Default port
const HTTP_PORT_NO: u16 = 80;

/// Default session timeout time in seconds
const TIMEOUT: u32 = 100;

// Default data expiry time in seconds
const EXPIRY: u32 = 3600 * 24 * 30;

// Default number of threads
const WORKERS: usize = 4;

/// Port for HTTP server
const HTTP_PORT: &str = "HTTP_PORT";

/// Path to the SQLite database file
const DATABASE_PATH: &str = "DATABASE_PATH";

/// Expiry time for data deletion in seconds
const DATA_EXPIRY: &str = "DATA_EXPIRY_SECONDS";

/// Login session timeout in seconds
const LOGIN_TIMEOUT: &str = "LOGIN_TIMEOUT_SECONDS";

/// Max number of threads
const WORKERS_MAX: &str = "WORKERS_MAX";

/// Actix state
struct AppState {
    pool: sqlx::sqlite::SqlitePool,
    timeout: u32,
    expiry: u32,
}

/// Login form
#[derive(serde::Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}

// Register device form
#[derive(serde::Deserialize)]
struct RegisterForm {
    did: u32,
}

// Send command to device form
#[derive(serde::Deserialize)]
struct CmdForm {
    did: u32,
    cmd: String,
}

// Upload data from device form
#[derive(serde::Deserialize)]
struct UploadForm {
    did: u32,
    data: String,
}

// Request download device log date range form
#[derive(serde::Deserialize)]
struct DownloadForm {
    start: u32,
    end: u32,
}

// Download device log
#[derive(sqlx::FromRow, serde::Serialize)]
struct Logs {
    did: u32,
    time: u32,
    data: String,
}

/// Current unix time
fn now() -> Result<u32, Error> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map_err(ErrorInternalServerError)?
        .as_secs() as u32)
}

/// Verify session login
async fn authenticate(session: &Session, pool: &sqlx::sqlite::SqlitePool) -> Result<u32, Error> {
    Ok(
        sqlx::query_scalar::<_, u32>("UPDATE login SET time = ? WHERE token = ? RETURNING uid")
            .bind(now()?) // 2038 bug
            .bind(
                session
                    .get::<u32>("token")?
                    .ok_or(ErrorUnauthorized("not logged in\n"))?,
            )
            .fetch_optional(pool)
            .await
            .map_err(ErrorInternalServerError)?
            .ok_or(ErrorUnauthorized("not logged in\n"))?,
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // process configuration
    let pool = if let Ok(file) = std::env::var(DATABASE_PATH) {
        SqlitePool::connect_with(
            SqliteConnectOptions::new()
                .filename(file)
                .create_if_missing(true),
        )
        .await
        .unwrap()
    } else {
        SqlitePool::connect("sqlite::memory:").await.unwrap()
    };
    let port = std::env::var(HTTP_PORT)
        .unwrap_or(HTTP_PORT_NO.to_string())
        .parse()
        .unwrap_or(HTTP_PORT_NO);
    let expiry = std::env::var(DATA_EXPIRY)
        .unwrap_or(EXPIRY.to_string())
        .parse()
        .unwrap_or(EXPIRY);
    let timeout = std::env::var(LOGIN_TIMEOUT)
        .unwrap_or(TIMEOUT.to_string())
        .parse()
        .unwrap_or(TIMEOUT);
    let workers = std::env::var(WORKERS_MAX)
        .unwrap_or(WORKERS.to_string())
        .parse()
        .unwrap_or(WORKERS);

    // Create database tables if don't exist
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS users (
            uid INTEGER PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            password VARCHAR(32) NOT NULL,
            time DATETIME NOT NULL,
            admin BOOLEAN
        );
        CREATE TABLE IF NOT EXISTS devices (
            did INTEGER PRIMARY KEY,
            uid INTEGER NOT NULL,
            cmd TEXT
        );
        CREATE TABLE IF NOT EXISTS login (
            token INTEGER PRIMARY KEY,
            uid INTEGER NOT NULL,
            time DATETIME NOT NULL
        );
        CREATE TABLE IF NOT EXISTS log (
            entry INTEGER PRIMARY KEY,
            did INTEGER NOT NULL,
            uid INTEGER NOT NULL,
            time DATETIME NOT NULL,
            data TEXT
        );",
    )
    .execute(&pool)
    .await
    .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                pool: pool.clone(),
                expiry: expiry,
                timeout: timeout,
            }))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(
                // create cookie based session middleware
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    cookie::Key::from(&[0; 64]),
                )
                .cookie_secure(false)
                .build(),
            )
            .route("/signup", web::post().to(signup))
            .route("/register", web::post().to(register))
            .route("/devices", web::get().to(devices))
            .route("/login", web::post().to(login))
            .route("/logout", web::get().to(logout))
            .route("/cmd", web::post().to(cmd))
            .route("/upload", web::post().to(upload))
            .route("/download", web::post().to(download))
            .route("/update", web::get().to(update))
            .service(
                Files::new("/", "./static")
                    .show_files_listing()
                    .index_file("index.html"),
            )
    })
    .workers(workers)
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

/// Create account form
async fn signup(
    session: Session,
    req: web::Json<LoginForm>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    if None == session.get::<u32>("token")? {
        let password = blake3::hash(req.password.as_bytes()).to_string();

        sqlx::query("INSERT INTO users (email, password, time, admin) VALUES (?, ?, ?, 0)")
            .bind(&req.email)
            .bind(&password)
            .bind(now()?) // 2038 bug
            .execute(&state.pool)
            .await
            .map_err(ErrorInternalServerError)?;
    }

    Ok(HttpResponse::Ok().json("signed up"))
}

/// Login page
async fn login(
    session: Session,
    req: web::Json<LoginForm>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    if None == session.get::<u32>("token")? {
        let (uid, password): (u32, String) =
            sqlx::query_as("SELECT uid, password FROM users WHERE email=?")
                .bind(&req.email)
                .fetch_one(&state.pool)
                .await
                .map_err(ErrorUnauthorized)?;

        if blake3::hash(req.password.as_bytes()).to_string() == password {
            let token: u32 =
                sqlx::query_scalar("INSERT INTO login (uid, time) VALUES (?, ?) RETURNING token")
                    .bind(&uid)
                    .bind(now()?) // 2038 bug
                    .fetch_one(&state.pool)
                    .await
                    .map_err(ErrorInternalServerError)?;

            session.insert("token", &token)?;
            session.insert("uid", &uid)?;
        } else {
            return Err(ErrorUnauthorized("Invalid username or password\n"));
        }
    }

    Ok(HttpResponse::Ok().json("logged in"))
}

/// Logout page
async fn logout(session: Session, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    if let Some(token) = session.get::<u32>("token")? {
        sqlx::query("DELETE FROM login WHERE token=?")
            .bind(&token)
            .execute(&state.pool)
            .await
            .map_err(ErrorInternalServerError)?;

        session.remove("token");
        session.remove("uid");

        Ok(HttpResponse::Ok().json("logout"))
    } else {
        Ok(HttpResponse::Ok().json("not logged in"))
    }
}

/// Register device page
async fn register(
    session: Session,
    req: web::Json<RegisterForm>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    sqlx::query("INSERT INTO devices (did, uid) VALUES (?, ?)")
        .bind(&req.did)
        .bind(authenticate(&session, &state.pool).await?)
        .execute(&state.pool)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json("registered"))
}

/// List devices page
async fn devices(session: Session, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(
        sqlx::query_scalar::<_, u32>("SELECT did FROM devices WHERE uid=?")
            .bind(authenticate(&session, &state.pool).await?)
            .fetch_all(&state.pool)
            .await
            .map_err(ErrorInternalServerError)?,
    ))
}

/// Send message to device page
async fn cmd(
    session: Session,
    req: web::Json<CmdForm>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let _ = authenticate(&session, &state.pool).await?;

    sqlx::query("UPDATE devices SET cmd=? WHERE did=?")
        .bind(&req.cmd)
        .bind(&req.did)
        .execute(&state.pool)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json("cmd requested"))
}

/// Device upload log and receive message page
async fn upload(
    session: Session,
    req: web::Json<UploadForm>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    sqlx::query("INSERT INTO log (did, uid, time, data) VALUES (?, ?, ?, ?)")
        .bind(&req.did)
        .bind(authenticate(&session, &state.pool).await?)
        .bind(now()?) // 2038 bug
        .bind(&req.data)
        .execute(&state.pool)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(
        sqlx::query_scalar::<_, String>("SELECT cmd FROM devices WHERE did=?")
            .bind(&req.did)
            .fetch_optional(&state.pool)
            .await
            .map_err(ErrorInternalServerError)?
            .unwrap_or_default(),
    ))
}

/// Download log page
async fn download(
    session: Session,
    req: web::Json<DownloadForm>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(
        sqlx::query_as::<_, Logs>(
            "SELECT did, time, data FROM log WHERE uid=? AND time>? AND time<?",
        )
        .bind(authenticate(&session, &state.pool).await?)
        .bind(&req.start)
        .bind(&req.end)
        .fetch_all(&state.pool)
        .await
        .map_err(ErrorInternalServerError)?,
    ))
}

/// Process timeouts and expiry page
async fn update(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let now = now()?;

    sqlx::query("DELETE FROM login WHERE time < ?")
        .bind(now - state.timeout) // 2038 bug
        .execute(&state.pool)
        .await
        .map_err(ErrorInternalServerError)?;

    sqlx::query("DELETE FROM log WHERE time < ?")
        .bind(now - state.expiry) // 2038 bug
        .execute(&state.pool)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json("updated"))
}
