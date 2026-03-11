# webThing

Proof of concept IoT REST HTTP JSON server using rust actix and SQLite through sqlx.

Objective
* High performance/Low footprint
* Docker scratch micros-service
* Direct light weight SQLite interfacing instead of slow socket sql server.
* Server should act as authenticator, data should be pass-through.
* Simple REST interface, not full RUSTFul.
* No TLS, expect to be run with Nginx or similar for TLS and static files.
* Provisional static files for testing.
* HURL for HTTP JSON API testing.
* SQLite replication available via sqlite3_rsync

## Docker

Build docker micro-service

`docker build -t webthing .`

Run docker micro-service

`docker run --rm -p 8080:8080 -e HTTP_PORT=8080 webthing`

## Environment variables

* HTTP_PORT - Port for HTTP server
* DATABASE_PATH - Path to the SQLite database file
* DATA_EXPIRY_SECONDS - Expiry time for data deletion in seconds
* LOGIN_TIMEOUT_SECONDS - Login session timeout in seconds
* WORKERS_MAX - Maximum number of threads

## TODO
* Timed update to process expiry and timeout.
* Edit account
* tests for DATA_EXPIRY_SECONDS, LOGIN_TIMEOUT_SECONDS and WORKERS_MAX

