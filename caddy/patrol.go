package patrol

import (
	"encoding/json"
	"io"
	"net/http"
	"net/url"
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

func removePatrolCookie(w http.ResponseWriter) {
	http.SetCookie(w, &http.Cookie{
		Name:   "patrol",
		Value:  "",
		Path:   "/",
		MaxAge: -1,
	})
}

func redirectToLogin(w http.ResponseWriter, r *http.Request) {
	http.Redirect(w, r, patrolBasePath+"/login?redirect_to="+url.QueryEscape(r.RequestURI), http.StatusSeeOther)
}

func (p Patrol) Authenticate(w http.ResponseWriter, r *http.Request) (caddyauth.User, bool, error) {
	// Extract the Patrol cookie
	cookie, err := r.Cookie("patrol")
	if err != nil {
		redirectToLogin(w, r)
		defer p.logger.Debug("No cookie found", zap.Error(err))
		return caddyauth.User{}, false, err
	}

	resp, err := p.client.Get("http://patrol:7288/session?id=" + cookie.Value)
	if err != nil {
		redirectToLogin(w, r)
		defer p.logger.Debug("Failed to check session with Patrol", zap.Error(err))
		return caddyauth.User{}, false, err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		removePatrolCookie(w)
		redirectToLogin(w, r)
		defer p.logger.Debug("Failed to read Patrol response", zap.Error(err))
		return caddyauth.User{}, false, err
	}

	if resp.StatusCode != http.StatusUnauthorized {
		redirectToLogin(w, r)
		defer p.logger.Debug("Unauthorized", zap.Int("status", resp.StatusCode))
		return caddyauth.User{}, false, err
	}

	var user user
	if err := json.Unmarshal(body, &user); err != nil {
		redirectToLogin(w, r)
		defer p.logger.Debug("Failed to unmarshal Patrol response", zap.Error(err))
		return caddyauth.User{}, false, err
	}

	r.Header.Set("X-Patrol", string(body))

	return caddyauth.User{ID: user.username}, true, nil
}

var (
	_ caddy.Provisioner       = (*Patrol)(nil)
	_ caddyauth.Authenticator = (*Patrol)(nil)
)
