FROM debian:buster-slim

ARG APP_NAME
ARG VERSION
ARG TARGET=x86_64-unknown-linux-gnu
ENV APP_NAME=${APP_NAME}

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
 && rm -rf /var/lib/apt/lists/*

RUN curl -L https://github.com/waltzofpearls/otto/releases/download/v${VERSION}/${APP_NAME}-${TARGET}-${VERSION}.tar.gz | tar xvz \
 && mv ${APP_NAME} /usr/local/bin

CMD ${APP_NAME}
