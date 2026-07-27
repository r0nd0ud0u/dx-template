# nginx setup (production VPS)

Production runs a **host-level nginx** (not dockerized) in front of the app
container, using `vps-nginx-site.conf` as the site config. The app itself
runs via `docker-compose.yml` (no nginx service — `docker-compose.nginx.yml`
is an alternate/unused dockerized-nginx variant, kept for reference only).

## Layout

- Redirect: `:80` → `:443` (HTTP→HTTPS)
- `/api/` → app container `:8080` — server functions (login, save, admin, ...)
  and the websocket (`/api/new-event`, see `src/websocket_handler/event.rs`)
- `/db/` → `sqlite-web` on `:8082` — protected by `auth_basic` + `.htpasswd`
- `/db/static/` → `sqlite-web` static assets, cached 7d
- `/` → app container `:8080` — static WASM/HTML/JS bundle (catch-all, must
  stay last)

## Firewall (your VPS provider's firewall/security-group panel)

Required open ports: **22** (SSH), **80** (HTTP, redirect + Let's Encrypt
HTTP-01 challenge), **443** (HTTPS). Everything else — notably `8082`
(sqlite-web) and `8080` (app) — should stay closed; they're only reached
internally via the nginx proxy.

## TLS (Let's Encrypt / certbot)

```bash
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d your-domain.example.com -d www.your-domain.example.com
```

Renewal is automatic (certbot installs a systemd timer/cron that renews
within 30 days of expiry) — no manual action needed. Verify it's actually
wired up:

```bash
systemctl list-timers | grep certbot
sudo certbot renew --dry-run
```

## `/db/` access (sqlite-web admin UI)

Protected by HTTP Basic Auth. Generate/update the password file:

```bash
sudo apt install apache2-utils
sudo htpasswd -c /etc/nginx/.htpasswd admin   # -c overwrites the file — omit it to add another user
```

## Rate limiting

`limit_req_zone`/`limit_conn_zone` (10 req/s per IP, burst 20, 20 concurrent
connections per IP) are applied on `/api/` and `/` — the two locations
proxying to the app. Only affects new HTTP requests/connections (login
attempts, page loads, the websocket handshake), not data already flowing
over an established websocket. Left off `/db/` (already gated by
`auth_basic`) and `/db/static/` (cached, low-risk).

If real users ever hit `429 Too Many Requests`, raise `rate=`/`burst=`
rather than removing the limit — that's a tuning signal, not a sign
something's broken.

## DDoS protection

Many VPS providers include network-level DDoS mitigation for volumetric
attacks (SYN/UDP floods, amplification) at no extra setup — check whether
yours does. It does **not** cover application-layer abuse (HTTP/websocket
floods from many IPs, brute force) — that's what the rate limiting above,
the app's own login/register limiter (`auth_rate_limit` in
`src/auth_manager/server_fn/auth.rs`, built on the `governor` crate), and
`fail2ban` (see `FAIL2BAN.md`) are for.

## Deploying a config change

```bash
sudo nginx -t                    # validate syntax
sudo systemctl reload nginx      # apply if valid
```
