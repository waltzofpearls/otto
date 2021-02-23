ARG APP_NAME=otto

FROM rust:1.50.0-alpine3.13 as builder

ARG APP_NAME
WORKDIR /app/${APP_NAME}

RUN apk add --no-cache -U musl-dev openssl openssl-dev

COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
 && echo 'fn main() {println!("if you see this, the build broke")}' > src/main.rs \
 && cargo build --release \
 && rm -f target/release/deps/${APP_NAME}*

COPY . .
RUN cargo build --release

FROM alpine:3.13

ARG APP_NAME
ENV APP_NAME=${APP_NAME}
WORKDIR /usr/local/bin

RUN apk add --no-cache -U openssl

COPY --from=builder /app/${APP_NAME}/target/release/${APP_NAME} ${APP_NAME}

CMD ${APP_NAME}
