# Patrol 💂

Patrol is a dead simple, no-fuss authentication and authorization service that integrates with the Caddy web server as a custom auth provider. It is super easy to setup (only a single Docker container).

## Request Flow 🆔

Patrol acts as a reverse proxy authentication layer:

1. User requests protected resource through Caddy
2. Caddy plugin validates Patrol session cookie
3. Valid sessions proceed with user info injected into headers which downstream services can use for authorization
4. Invalid sessions redirect to Patrol login page
5. After login, user is redirected back to original resource

This repository contains both Patrol itself and the Caddy plugin.

## Quick Start 🏃

### Using Docker

```bash
# Clone the repository
git clone <repository-url>
cd patrol

# Build and run with Docker
docker build -t patrol .
docker run -p 7287:7287 -p 7288:7288 patrol
```

### Configuration 🔧

Example `docker-compose.yaml` and `Caddyfile` might look something like this (with configuration in the `data` directory):

```yaml
# docker-compose.yaml
name: thing

services:
  caddy:
    # For this, see https://hub.docker.com/_/caddy
    container_name: caddy
    build:
      context: ./caddy
    restart: unless-stopped
    cap_add:
      - NET_ADMIN
    ports:
      - "80:80"
      - "443:443"
      - "443:443/udp"
    volumes:
      - "./data/caddy/:/etc/caddy/"
      - "caddy_data:/data/"
      - "caddy_config:/config/"
    depends_on:
      patrol:
        condition: service_healthy

  patrol:
    container_name: patrol
    build:
      context: .
    restart: unless-stopped
    environment:
      RUST_LOG: trace
      RUST_BACKTRACE: 1
    volumes:
      # Volume for the SQLite database
      - "./data/patrol/:/app/data/"

volumes:
  caddy_data:
    external: true
  caddy_config:
    external: true
```

```Caddyfile
# data/caddy/Caddyfile
{
	order patrol before basicauth
}

:80 {
	handle_path /patrol* {
		@auth {
			not path /login /logout /register /register/*
		}

		patrol @auth
		reverse_proxy patrol:7287
	}

	handle_path /your-service* {
		patrol

		# Rest of your configuration
	}
}
```

## Development 🧑‍💻

The goal of the project was to create an OAuth 2.0 server for self-hosters. It
turned out that OAuth is mostly focused on enterprise and is thus too
complicated for us, hobbyists. As time progressed, I stripped parts of the OAuth
spec out and pretty much ended up with session auth. It's much simpler, we don't
need separated services, and works just as fine (if not better). Below, you'll
find the structure I've arrived at so far.

### Project Structure

```
├── src/               # Rust application source
│   ├── models/        # Database entities
│   ├── pages/         # HTTP route handlers
│   ├── session/       # Session management
│   └── crypto/        # Security utilities
├── caddy/             # Caddy authentication plugin
├── db/                # Database schema and migrations
├── templates/         # Web interface templates
├── static/            # Frontend assets
└── Dockerfile         # Container configuration
```

### Technology Stack

- Rust
  - Poem
  - SeaORM
- SQLite
- Caddy
- dbmate

### Database Migrations

```bash
# Create new migration
dbmate new migration_name

# Apply migrations
dbmate up

# Rollback migration
dbmate down

# Drop database
dbmate drop
```
