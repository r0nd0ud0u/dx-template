# fail2ban setup (production VPS)

Nginx's `limit_req`/`limit_conn` (see `NGINX.md`) and the app's own
`/api/user/login`+`/api/register` rate limiter (`auth_rate_limit` in
`src/auth_manager/server_fn/auth.rs`) throttle abuse in real time, but neither
*bans* an offending IP — they just slow it down, request after request,
forever. fail2ban watches the logs those two layers already produce and
escalates repeat offenders to an actual firewall ban.

## Install

```bash
sudo apt install fail2ban
```

## Layer 1 — nginx's own rate-limit rejections

Whenever nginx's `limit_req_zone`/`limit_conn_zone` (configured in
`vps-nginx-site.conf` for `/api/` and `/`) rejects a request, it logs a
`limiting requests` line to `/var/log/nginx/error.log` with the client IP.
fail2ban ships a filter for this out of the box — no custom regex needed.

Create/edit `/etc/fail2ban/jail.local` (never edit `jail.conf` directly —
`jail.local` is read as an override on top of it):

```ini
[DEFAULT]
# Repeat offenders get banned longer each time instead of a flat bantime.
bantime.increment = true
bantime.factor = 2
bantime.maxtime = 1w

[nginx-limit-req]
enabled  = true
filter   = nginx-limit-req
port     = http,https
logpath  = /var/log/nginx/error.log
findtime = 600      # look at the last 10 minutes
maxretry = 20       # ban after 20 rejected/throttled requests in that window
bantime  = 3600     # 1 hour base ban (see bantime.increment above)
```

## Layer 2 — login/register specifically

`limit_req` on `/api/` is a broad 10 req/s bucket shared by every API call —
it won't necessarily catch someone slow-rolling login/register attempts just
under that threshold but still tripping the app's own stricter per-endpoint
limiter (3 back-to-back, then 1/12s — see `auth_rate_limit`). That returns
HTTP `429`, which nginx's *access* log does record, so a small custom filter
catches it:

`/etc/fail2ban/filter.d/dxtemplate-auth.conf`:

```ini
[Definition]
failregex = ^<HOST> -.*"(POST) /api/(user/login|register) HTTP/.*" 429
ignoreregex =
```

Append to `/etc/fail2ban/jail.local`:

```ini
[dxtemplate-auth]
enabled  = true
filter   = dxtemplate-auth
port     = http,https
logpath  = /var/log/nginx/access.log
findtime = 600
maxretry = 3
bantime  = 3600
```

## Apply and verify

```bash
sudo systemctl restart fail2ban
sudo fail2ban-client status
sudo fail2ban-client status nginx-limit-req
sudo fail2ban-client status dxtemplate-auth
```

Test the ban/unban mechanics without waiting for a real trigger:

```bash
sudo fail2ban-client set dxtemplate-auth banip <test-ip>
sudo fail2ban-client set dxtemplate-auth unbanip <test-ip>
```

## Why two layers instead of one

- **nginx-limit-req** catches broad floods/scraping against any API endpoint.
- **dxtemplate-auth** catches credential-stuffing/account-spam specifically,
  before it'd ever be loud enough to trip the broad `/api/` limit.

Both read from logs nginx already writes — no extra logging setup required
on the app side.
