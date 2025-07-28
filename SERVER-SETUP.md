<!-- TOC -->

- [Building server](#building-server)
- [Single-domain nginx configuration](#a-single-domain-nginx-configuration)
- [Multi-domain nginx configuration](#b-multi-domain-nginx-configuration)

<!-- /TOC -->

# Server setup

Requirements:

- [Rust](https://rust-lang.org)
- [nginx](https://nginx.org/en)

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

Server URL will be `https://axel.assasans.dev/static/`.

Example nginx configuration:

```nginx
server {
  listen       443 ssl;
  server_name  axel.assasans.dev;

  ssl_certificate /path/to/axel/sesisoft.com.crt;
  ssl_certificate_key /path/to/axel/sesisoft.com.key;

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

Server URL will be `https://static.konosuba.local/`.

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
