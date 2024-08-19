package patrol

import (
	"context"
	"fmt"
	"net/http"

	"github.com/caddyserver/caddy/v2"
	"github.com/caddyserver/caddy/v2/modules/caddyhttp/caddyauth"
	"github.com/lestrrat-go/jwx/jwt"
	"github.com/lestrrat-go/jwx/v2/jwk"
	"go.uber.org/zap"
)

const jwkUrl = "http://patrol:7287/.well-known/jwks.json"

func init() {
	caddy.RegisterModule(Patrol{})
	fmt.Println("Patrol module registered")
}

type Patrol struct {
	jwkSet *jwk.Set

	logger *zap.Logger
}

func (p *Patrol) Provision(ctx caddy.Context) error {
	p.logger = ctx.Logger()
	return nil
}

func (p *Patrol) Validate() error {
	jwkSet, err := jwk.Fetch(context.Background(), jwkUrl)
	if err != nil {
		return err
	}

	p.jwkSet = &jwkSet
	return nil
}

func (Patrol) CaddyModule() caddy.ModuleInfo {
	return caddy.ModuleInfo{
		ID:  "http.authentication.providers.patrol",
		New: func() caddy.Module { return new(Patrol) },
	}
}

func (p Patrol) Authenticate(w http.ResponseWriter, r *http.Request) (caddyauth.User, bool, error) {
	cookie, err := r.Cookie("patrol")
	if err != nil {
		return caddyauth.User{}, false, nil
	}

	token, err := jwt.ParseString(cookie.Value, jwt.WithIssuer("patrol"))
	if err != nil {
		return caddyauth.User{}, false, err
	}

	p.logger.Info("Token", zap.Any("token", token))

	http.Redirect(w, r, "/patrol/login", http.StatusSeeOther)

	return caddyauth.User{}, true, nil
}

func (p *Patrol) Error(err error) {
	p.logger.Error("Patrol error", zap.Error(err))
}

var (
	_ caddy.Provisioner       = (*Patrol)(nil)
	_ caddy.Validator         = (*Patrol)(nil)
	_ caddyauth.Authenticator = (*Patrol)(nil)

	// _ httprc.ErrSink = (*Patrol)(nil)
)
