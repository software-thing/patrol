package patrol

import (
	"github.com/caddyserver/caddy/v2"
	"github.com/caddyserver/caddy/v2/caddyconfig"
	"github.com/caddyserver/caddy/v2/caddyconfig/httpcaddyfile"
	"github.com/caddyserver/caddy/v2/modules/caddyhttp"
	"github.com/caddyserver/caddy/v2/modules/caddyhttp/caddyauth"
)

func init() {
	httpcaddyfile.RegisterHandlerDirective("patrol", func(h httpcaddyfile.Helper) (caddyhttp.MiddlewareHandler, error) {
		var p Patrol

		return caddyauth.Authentication{
			ProvidersRaw: caddy.ModuleMap{
				"patrol": caddyconfig.JSON(p, nil),
			},
		}, nil
	})
}
