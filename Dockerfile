FROM rust:1.93.1-alpine3.23 AS builder

RUN apk add --no-cache musl-dev sqlite-static sqlite-dev 

WORKDIR /wd
COPY . /wd
RUN cargo build --bins --release

FROM scratch

COPY --from=builder /wd/target/release/webThing /
COPY static /static
CMD ["/webThing"]
