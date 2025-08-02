<!-- TOC -->

- [Configuration](#configuration)
- [Building server](#building-server)
  - [Protocol and encryption](#protocol-and-encryption)
- [A. Single-domain nginx configuration](#a-single-domain-nginx-configuration)
- [B. Multi-domain nginx configuration](#b-multi-domain-nginx-configuration)

<!-- /TOC -->
# Server setup

Requirements:

- Linux (for running database migration script, you may run them manually on any OS)
- [Rust](https://rust-lang.org)
- [nginx](https://nginx.org/en)
- [PostgreSQL](https://postgresql.org/)

See [DATABASE-SETUP.md](DATABASE-SETUP.md) for instructions on how to set up the PostgreSQL database.

## Configuration

Edit `config/default.toml` or create `config/local.toml` to edit your configuration.

At the very least, you need to set `static-server.public-url` and `api-server.public-url` according to your nginx configuration.

## Building server

`RUST_LOG=info cargo run`

The game uses HTTPS protocol for all requests, so we need to generate our TLS certificate.
Axel itself does not handle HTTPS, you need to use a reverse proxy server like nginx.

### Protocol and encryption

The game uses JSON for responses and `application/x-www-form-urlencoded` for requests.

Server signs all API responses with JWT RS256 (RSA-1024 key).
**This key pair is not related to the TLS certificate.**
If you want to generate a new RSA key pair:

* run `openssl genpkey -algorithm RSA -out key.pem -pkeyopt rsa_keygen_bits:1024`;
* then run `openssl rsa -in key.pem -pubout -out pubkey.pem` to get the public key.

## A. Single-domain nginx configuration

* Static public URL (and URL for patching) will be `https://axel.assasans.dev/static/`.
* API public URL will be `https://axel.assasans.dev/api/`.

Example nginx configuration:

```nginx
server {
  listen       443 ssl;
  server_name  axel.assasans.dev;

  ssl_certificate /path/to/axel/axel.assasans.dev.crt;
  ssl_certificate_key /path/to/axel/axel.assasans.dev.key;

  proxy_set_header Host $host;
  proxy_set_header X-Real-IP $remote_addr;
  proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
  proxy_set_header X-Forwarded-Proto $scheme;

  location /static/ {
    proxy_pass http://127.0.0.1:2021/;
  }

  location /api/ {
    proxy_pass http://127.0.0.1:2020/;
  }
}
```

## B. Multi-domain nginx configuration

* Static public URL (and URL for patching) will be `https://static.konosuba.local/`.
* API public URL will be `https://api.konosuba.local/`.

Example nginx configuration:

```nginx
server {
  listen 443 ssl;
  server_name api.konosuba.local;

  ssl_certificate /path/to/axel/sesisoft.com.crt;
  ssl_certificate_key /path/to/axel/sesisoft.com.key;

  location / {
    proxy_pass http://127.0.0.1:2020;

    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
  }
}

server {
  listen 443 ssl;
  server_name static.konosuba.local;

  ssl_certificate /path/to/axel/sesisoft.com.crt;
  ssl_certificate_key /path/to/axel/sesisoft.com.key;

  location / {
    proxy_pass http://127.0.0.1:2021;

    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
  }
}
```
