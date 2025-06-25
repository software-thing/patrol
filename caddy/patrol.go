package patrol

import (
	"encoding/json"
	"io"
	"net/http"
	"os"

	"github.com/caddyserver/caddy/v2"
	"github.com/caddyserver/caddy/v2/modules/caddyhttp/caddyauth"
	"go.uber.org/zap"
)

const jwkUrl = "http://patrol:7287/.well-known/jwks.json"
const redisUrl = "redis://valkey:6379"

var patrolBasePath = "/patrol"

func init() {
	if path := os.Getenv("PATROL_BASE_PATH"); path != "" {
		patrolBasePath = path
	}

	caddy.RegisterModule(Patrol{})
}

type Patrol struct {
	client *http.Client
	logger *zap.Logger
}

func (p *Patrol) Provision(ctx caddy.Context) error {
	p.client = &http.Client{}
	p.logger = ctx.Logger()
	return nil
}

func (Patrol) CaddyModule() caddy.ModuleInfo {
	return caddy.ModuleInfo{
		ID:  "http.authentication.providers.patrol",
		New: func() caddy.Module { return new(Patrol) },
	}
}

type user struct {
	username string
}

func (p Patrol) Authenticate(w http.ResponseWriter, r *http.Request) (caddyauth.User, bool, error) {
	// Extract the Patrol cookie
	cookie, err := r.Cookie("patrol")
	if err != nil {
		http.Redirect(w, r, patrolBasePath+"/login", http.StatusSeeOther)
		defer p.logger.Error("No cookie found", zap.Error(err))
		return caddyauth.User{}, false, err
	}

	resp, err := p.client.Get("http://patrol:7288/session?id=" + cookie.Value)
	if err != nil {
		http.Redirect(w, r, patrolBasePath+"/login", http.StatusSeeOther)
		p.logger.Error("Failed to check session with Patrol", zap.Error(err))
		return caddyauth.User{}, false, err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		http.Redirect(w, r, patrolBasePath+"/login", http.StatusSeeOther)
		p.logger.Error("Failed to read Patrol response", zap.Error(err))
		return caddyauth.User{}, false, err
	}

	var user user
	json.Unmarshal(body, &user)

	r.Header.Set("X-Patrol", string(body))

	return caddyauth.User{ID: user.username}, true, nil
}

var (
	_ caddy.Provisioner       = (*Patrol)(nil)
	_ caddyauth.Authenticator = (*Patrol)(nil)
)
