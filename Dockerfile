FROM rust:1-alpine AS builder

WORKDIR /usr/src/patrol

RUN apk add --no-cache musl-dev

COPY src/ src/
COPY Cargo.toml Cargo.lock ./

RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/src/patrol/target \
  cargo install --path .

FROM alpine:3

# Install dbmate
RUN apk add --no-cache curl
RUN curl -fsSL -o /usr/local/bin/dbmate https://github.com/amacneil/dbmate/releases/latest/download/dbmate-linux-amd64
RUN chmod +x /usr/local/bin/dbmate

RUN ls -la /usr/local/bin

WORKDIR /app

COPY bin/ bin/
COPY db/ db/
COPY templates/ templates/

VOLUME ["/app/keys/", "/app/.env"]
EXPOSE 7287

COPY --from=builder /usr/local/cargo/bin/patrol /usr/local/bin/patrol

CMD "/app/bin/run.sh"
