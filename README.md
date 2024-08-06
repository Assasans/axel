<div align="center">

# Axel — a private server for *KonoSuba: Fantastic Days*

</div>

See [DUMPING.md](DUMPING.md) for instructions on how to inspect the game code yourself.

### Current progress

The first scene ("And so the Adventure Begins!") loads and goes up to the first battle tutorial.

![](https://files.catbox.moe/xvvt4z.png)

## Setup

**Server requirements:**

- [Rust](https://rust-lang.org)
- [nginx](https://nginx.org/en)

**Client requirements:**

- Linux machine (preferably [Arch Linux](https://archlinux.org/), `x86_64` or `arm64-v8a`), [WSL](https://learn.microsoft.com/en-us/windows/wsl/install) is not tested
- [Waydroid](https://waydro.id)
- [Rust](https://rust-lang.org) — to build the [RSA patcher](rsa-patcher)

### Building server

`RUST_LOG=info cargo run`

### RSA signing issue

The game signs API responses with JWT RS256 (RSA-1024 key).
This server uses its own key pair, so the public key must be replaced in the client.

Static key replacement is not possible due to the LIAPP anti-tampering system (please comment on [this issue](https://github.com/Assasans/axel/issues/1) if you know how to work around it).
The current solution is to dynamically replace the key in the process memory after the game has been started.

If you want to generate a new RSA key pair — run `openssl genpkey -algorithm RSA -out key.pem -pkeyopt rsa_keygen_bits:1024`.

### Waydroid

Currently, the only supported way to run the game is through [Waydroid](https://wiki.archlinux.org/title/Waydroid).
Contributions with tutorials for real devices are welcome.

1. You will need a kernel with the `binder` module, see your distribution documentation for more information.
2. Initialize the Waydroid image — `sudo waydroid init -s GAPPS`
3. Start Waydroid service — `sudo systemctl start waydroid-container`
4. Install `libhoudini` if you are not on an ARM host: https://github.com/casualsnek/waydroid_script
5. Add the following properties to `/var/lib/waydroid/waydroid.cfg` ([source](https://github.com/waydroid/waydroid/issues/1060)):
```
[properties]
ro.product.brand=google
ro.product.manufacturer=Google
ro.system.build.product=redfin
ro.product.name=redfin
ro.product.device=redfin
ro.product.model=Pixel 5
ro.system.build.flavor=redfin-user
ro.build.fingerprint=google/redfin/redfin:11/RQ3A.211001.001/eng.electr.20230318.111310:user/release-keys
ro.system.build.description=redfin-user 11 RQ3A.211001.001 eng.electr.20230318.111310 release-keys
ro.bootimage.build.fingerprint=google/redfin/redfin:11/RQ3A.211001.001/eng.electr.20230318.111310:user/release-keys
ro.build.display.id=google/redfin/redfin:11/RQ3A.211001.001/eng.electr.20230318.111310:user/release-keys
ro.build.tags=release-keys
ro.build.description=redfin-user 11 RQ3A.211001.001 eng.electr.20230318.111310 release-keys
ro.vendor.build.fingerprint=google/redfin/redfin:11/RQ3A.211001.001/eng.electr.20230318.111310:user/release-keys
ro.vendor.build.id=RQ3A.211001.001
ro.vendor.build.tags=release-keys
ro.vendor.build.type=user
ro.odm.build.tags=release-keys
ro.adb.secure = 1
ro.debuggable = 0
```

6. Update configuration — `waydroid upgrade --offline`
7. Start Waydroid session — `waydroid session start` for Wayland users, `cage waydroid session start` for X11 users
8. Launch a GUI — `waydroid show-full-ui`
9. Install the game from Google Play or APK:
   1. Download and extract latest XAPK from [APKPure](https://apkpure.com/konosuba-fantastic-days/com.nexon.konosuba/download)
   2. Install APKs — `adb install-multiple com.nexon.konosuba.apk config.arm64_v8a.apk`

### Redirecting DNS

Waydroid uses `dnsmasq`, so we use it to redirect `web-prd-wonder.sesisoft.com` to our server's IP.

#### (Recommended) Redirect for Waydroid only

```shell
# Use custom dnsmasq config
sudo sed -i 's|LXC_DHCP_CONFILE=""|LXC_DHCP_CONFILE="/var/lib/waydroid/lxc/waydroid/dnsmasq.conf"|' /usr/lib/waydroid/data/scripts/waydroid-net.sh

# Enter the server's IP here, it must be accessible from Waydroid.
local_ip="192.168.1.102"

# Redirect API host to our IP
cat <<EOF | sudo tee -a /var/lib/waydroid/lxc/waydroid/dnsmasq.conf
address=/web-prd-wonder.sesisoft.com/$local_ip
EOF
```

#### Redirect for both Waydroid and your host

Note: You need to make sure that Waydroid will not run its own `dnsmasq` instance.

```shell
# Enter the server's IP here, it must be accessible from Waydroid.
local_ip="192.168.1.102"

cat <<EOF | sudo tee -a /etc/resolvconf.conf
name_servers=127.0.0.1 # Use dnsmasq

dnsmasq_conf=/etc/dnsmasq-conf.conf
dnsmasq_resolv=/etc/dnsmasq-resolv.conf
EOF

sudo resolvconf -u

cat <<EOF | sudo tee -a /etc/dnsmasq.conf
conf-file=/etc/dnsmasq-conf.conf
resolv-file=/etc/dnsmasq-resolv.conf

address=/web-prd-wonder.sesisoft.com/$local_ip
EOF

sudo systemctl restart dnsmasq
```

### Installing TLS certificate

The game uses HTTPS protocol for all requests, so we need to generate our TLS certificate.
Axel itself does not handle HTTPS, you need to use a proxy server like nginx.

```shell
openssl req -x509 -newkey rsa:4096 -sha256 -days 3650 \
  -nodes -keyout sesisoft.com.key -out sesisoft.com.crt -subj "/CN=sesisoft.com" \
  -addext "subjectAltName=DNS:sesisoft.com,DNS:web-prd-wonder.sesisoft.com,DNS:static-prd-wonder.sesisoft.com"
openssl x509 -in sesisoft.com.crt -out sesisoft.com.pem -outform PEM

# Install certificate on Waydroid
hash=$(openssl x509 -subject_hash_old -in sesisoft.com.pem | head -1)
sudo cp sesisoft.com.pem /var/lib/waydroid/overlay/system/etc/security/cacerts/$hash.0
sudo chmod 644 /var/lib/waydroid/overlay/system/etc/security/cacerts/$hash.0
```

Example nginx configuration:

```nginx
server {
  listen 443 ssl;
  server_name web-prd-wonder.sesisoft.com;
  
  ssl_certificate /path/to/axel/sesisoft.com.crt;
  ssl_certificate_key /path/to/axel/sesisoft.com.key;
  
  # SSL settings for better security
  ssl_protocols TLSv1.2 TLSv1.3;
  ssl_ciphers HIGH:!aNULL:!MD5;
  ssl_prefer_server_ciphers on;
  
  location / {
    # Proxy pass to the backend HTTP server
    proxy_pass http://127.0.0.1:2020;

    # Proxy settings to pass original client request details
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
  }
}
```

### Starting the game

1. Start `KonoSuba: FD` in Waydroid and wait for the title screen to appear ("Connection Error" alert will appear — ignore it).
2. Build the [RSA key patcher](rsa-patcher) — `cargo build --release`.
3. And run it — `sudo RUST_LOG=info ./target/release/axel-rsa-patcher $(pidof com.nexon.konosuba)`.
4. Press OK on the error alert, the game should now work.

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or https://apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
