FROM alpine:3.13

ARG APP_NAME
ARG VERSION
ARG TARGET=x86_64-unknown-linux-musl
ENV APP_NAME=${APP_NAME}

RUN apk add --no-cache -U openssl curl

RUN curl -L https://github.com/waltzofpearls/otto/releases/download/v${VERSION}/${APP_NAME}-${TARGET}-${VERSION}.tar.gz | tar xvz \
 && mv ${APP_NAME} /usr/local/bin

CMD ${APP_NAME}
