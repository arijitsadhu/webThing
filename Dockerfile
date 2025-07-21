FROM rust:1.88.0-alpine3.22 AS builder

RUN apk add --no-cache musl-dev sqlite-static sqlite-dev

WORKDIR /wd
COPY . /wd
RUN cargo build --bins --release

FROM scratch

COPY --from=builder /wd/target/release/webThing /
CMD ["./webThing"]
