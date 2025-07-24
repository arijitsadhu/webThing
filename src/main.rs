use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::{
    App, Error, HttpRequest, HttpResponse, HttpServer, Result, cookie::Key,
    error::ErrorInternalServerError, error::ErrorUnauthorized, http::header::LOCATION,
    middleware::Logger, web,
};

const TIMEOUT: u32 = 100;
const WORKERS: usize = 4;

struct AppState {
    pool: sqlx::sqlite::SqlitePool,
}

#[derive(serde::Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}

#[derive(serde::Deserialize)]
struct RegisterForm {
    did: u32,
}

#[derive(serde::Deserialize)]
struct CmdForm {
    did: u32,
    cmd: String,
}

#[derive(serde::Deserialize)]
struct UploadForm {
    did: u32,
    data: String,
}

#[derive(serde::Deserialize)]
struct DownloadForm {
    start: u32,
    end: u32,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct Logs {
    did: u32,
    time: u32,
    data: String,
}

fn now() -> Result<u32, Error> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map_err(ErrorInternalServerError)?
        .as_secs() as u32)
}

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

    let pool = match std::env::var("DATABASE_FILE") {
        Ok(file) => sqlx::sqlite::SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(file)
                .create_if_missing(true),
        )
        .await
        .unwrap(),
        Err(_) => sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap(),
    };

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
            uid INTEGER NOT NULL
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
        ); 
        CREATE TABLE IF NOT EXISTS cmd (
            did INTEGER PRIMARY KEY,
            cmd TEXT
        );",
    )
    .execute(&pool)
    .await
    .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { pool: pool.clone() }))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(
                // create cookie based session middleware
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
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
    })
    .workers(WORKERS)
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

async fn signup(
    session: Session,
    req: web::Json<LoginForm>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    if let None = session.get::<u32>("token")? {
        let password = blake3::hash(req.password.as_bytes()).to_string();

        sqlx::query(
            "INSERT INTO users (email, password, time, admin)
                    VALUES (?, ?, ?, 0)",
        )
        .bind(&req.email)
        .bind(&password)
        .bind(now()?) // 2038 bug
        .execute(&state.pool)
        .await
        .map_err(ErrorInternalServerError)?;
    }

    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .body("signed up\n"))
}

async fn login(
    session: Session,
    req: web::Json<LoginForm>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    if let None = session.get::<u32>("token")? {
        let (uid, password): (u32, String) =
            sqlx::query_as("SELECT uid, password FROM users WHERE email=?")
                .bind(&req.email)
                .fetch_one(&state.pool)
                .await
                .map_err(ErrorUnauthorized)?;

        if blake3::hash(req.password.as_bytes()).to_string() == password {
            let token: u32 = sqlx::query_scalar(
                "INSERT INTO login (uid, time)
                    VALUES (?, ?) RETURNING token",
            )
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

    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .body("logged in\n"))
}

async fn logout(session: Session, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    if let Some(token) = session.get::<u32>("token")? {
        sqlx::query("DELETE FROM login WHERE token=?")
            .bind(&token)
            .execute(&state.pool)
            .await
            .map_err(ErrorInternalServerError)?;

        session.remove("token");
        session.remove("uid");

        Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/"))
            .body("logout\n"))
    } else {
        Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/"))
            .body("not logged in\n"))
    }
}

async fn register(
    session: Session,
    req: web::Json<RegisterForm>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    sqlx::query(
        "INSERT INTO devices (did, uid)
                    VALUES (?, ?)",
    )
    .bind(&req.did)
    .bind(authenticate(&session, &state.pool).await?)
    .execute(&state.pool)
    .await
    .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("uploaded\n"))
}

async fn devices(session: Session, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(
        sqlx::query_scalar::<_, u32>("SELECT did FROM devices WHERE uid=?")
            .bind(authenticate(&session, &state.pool).await?)
            .fetch_all(&state.pool)
            .await
            .map_err(ErrorInternalServerError)?,
    ))
}

async fn cmd(
    session: Session,
    req: web::Json<CmdForm>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let _ = authenticate(&session, &state.pool).await?;

    sqlx::query(
        "INSERT INTO cmd (cmd)
                    VALUES (?) where did=?",
    )
    .bind(&req.cmd)
    .bind(&req.did)
    .execute(&state.pool)
    .await
    .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("cmd requested\n"))
}

async fn upload(
    session: Session,
    req: web::Json<UploadForm>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    sqlx::query(
        "INSERT INTO log (did, uid, time, data)
                    VALUES (?, ?, ?, ?)",
    )
    .bind(&req.did)
    .bind(authenticate(&session, &state.pool).await?)
    .bind(now()?) // 2038 bug
    .bind(&req.data)
    .execute(&state.pool)
    .await
    .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(
        sqlx::query_scalar::<_, String>("SELECT cmd FROM cmd WHERE did=?")
            .bind(&req.did)
            .fetch_one(&state.pool)
            .await
            .map_err(ErrorInternalServerError)?,
    ))
}

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

async fn update(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    sqlx::query("DELETE FROM login WHERE time < ?")
        .bind(now()? - TIMEOUT) // 2038 bug
        .execute(&state.pool)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .body("updated\n"))
}
