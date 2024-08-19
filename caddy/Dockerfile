FROM caddy:builder-alpine AS builder

RUN --mount=type=bind,target=/usr/src/patrol/,rw --mount=type=cache,target=/go/pkg/mod/ --mount=type=cache,target=/root/.cache/go-build/ \
  xcaddy build \
  --with github.com/software-thing/patrol=/usr/src/patrol/


FROM caddy:alpine

COPY --from=builder /usr/bin/caddy /usr/bin/caddy    
