# webThing

Proof of concept IoT REST HTTP JSON server using rust actix and SQLite through sqlx running as a docker micro-service

Objective
* High performance/Low footprint
* Docker scratch micro-service
* Direct light weight SQLite interfacing instead of slow socket sql server.
* Server should act as authenticator, data should be pass-through.
* Simple REST JSON interface, not full RUSTFul.
* No TLS, expect to be run with Nginx or similar for TLS and static files.
* Provisional static files for testing.
* HURL for HTTP JSON API testing.
* SQLite replication available via sqlite3_rsync

## API

Signup user
```
POST /signup
{
    "email": "foo@example.com",
    "password": "bar"
}
```

Login user
```
POST /login
{
    "email": "foo@example.com",
    "password": "bar"
}
```

Register device
```
POST /register
{
    "did": 1234
}
```

Send message to device
```
POST /cmd
{
    "did": 1234,
    "cmd": "foo"
}
```

Device upload log and receive message
```
POST /upload
{
    "did": 1234,
    "data": "bar"
}


{
    "foo"
}
```

Download logs with date range
```
POST /download
{
    "start": 0,
    "end": 4294967295
}


{
    {
        did: 1234,
        time: 1773333069,
        data: "bar"
    },
    {
        did: 1234,
        time: 1773333080,
        data: "bared"
    }
    ...
}
```

List devices
```
GET /devices


{
    1234,
    2345
    ...
}

```

Logout user
```
GET /logout
```

## Docker

Build docker micro-service

`docker build -t webthing .`

Run docker micro-service

`docker run --rm -p 8080:8080 -e HTTP_PORT=8080 webthing`

## Environment variables

* HTTP_PORT - Port for HTTP server
* DATABASE_PATH - Path to the SQLite database file. If not specied it will use RAM database.
* DATA_EXPIRY_SECONDS - Expiry time for log deletion in seconds
* LOGIN_TIMEOUT_SECONDS - Login session timeout in seconds
* WORKERS_MAX - Maximum number of threads

## TODO
* Timed update to process expiry and timeout.
* Edit account
* tests for DATA_EXPIRY_SECONDS, LOGIN_TIMEOUT_SECONDS and WORKERS_MAX

