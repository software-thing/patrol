package patrol

import (
	"context"
	"encoding/base64"
	"errors"
	"fmt"
	"net/http"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/caddyserver/caddy/v2"
	"github.com/caddyserver/caddy/v2/modules/caddyhttp/caddyauth"
	"github.com/lestrrat-go/jwx/v2/jwk"
	"github.com/lestrrat-go/jwx/v2/jwt"
	"github.com/redis/go-redis/v9"
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
	jwkSet *jwk.Set

	redis  *redis.Client
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

	redisOpts, err := redis.ParseURL(redisUrl)
	if err != nil {
		return err
	}

	p.redis = redis.NewClient(redisOpts)

	return nil
}

func (Patrol) CaddyModule() caddy.ModuleInfo {
	return caddy.ModuleInfo{
		ID:  "http.authentication.providers.patrol",
		New: func() caddy.Module { return new(Patrol) },
	}
}

func (p Patrol) Authenticate(w http.ResponseWriter, r *http.Request) (caddyauth.User, bool, error) {
	// Extract the Patrol cookie
	cookie, err := r.Cookie("patrol")
	if err != nil {
		http.Redirect(w, r, patrolBasePath+"/login", http.StatusSeeOther)
		defer p.logger.Error("No cookie found", zap.Error(err))
		return caddyauth.User{}, false, nil
	}

	// Validate the JWT
	token, err := jwt.ParseString(
		cookie.Value,
		jwt.WithIssuer("patrol"),
		jwt.WithKeySet(*p.jwkSet),
		jwt.WithAcceptableSkew(time.Minute),
	)
	if err != nil {
		http.Redirect(w, r, patrolBasePath+"/login", http.StatusSeeOther)
		defer p.logger.Error("Invalid token", zap.Error(err))
		return caddyauth.User{}, false, nil
	}

	// Check Redis whether the key is still active
	key := fmt.Sprintf("token:%s:%s", token.Subject(), token.JwtID())
	exists, err := p.redis.Exists(context.Background(), key).Result()
	if err != nil {
		return caddyauth.User{}, false, err
	}

	if exists != 1 {
		return caddyauth.User{}, false, nil
	}

	base64URLPayload := strings.Split(cookie.Value, ".")
	if len(base64URLPayload) != 3 {
		return caddyauth.User{}, false, errors.New("invalid token")
	}

	payload, err := base64.RawURLEncoding.DecodeString(base64URLPayload[1])
	if err != nil {
		return caddyauth.User{}, false, err
	}

	var dataBuilder strings.Builder
	for _, b := range string(payload) {
		if b < 128 {
			dataBuilder.WriteRune(b)
		} else {
			dataBuilder.WriteString("\\u" + fmt.Sprintf("%04s", strconv.FormatInt(int64(b), 16)))
		}
	}

	r.Header.Set("X-Patrol", dataBuilder.String())

	return caddyauth.User{ID: token.Subject()}, true, nil
}

var (
	_ caddy.Provisioner       = (*Patrol)(nil)
	_ caddy.Validator         = (*Patrol)(nil)
	_ caddyauth.Authenticator = (*Patrol)(nil)
)
