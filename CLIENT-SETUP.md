<!-- TOC -->

- [Stub APK (recommended)](#stub-apk-recommended)
- [Original APK with Waydroid](#original-apk-with-waydroid)
  - [A. Without your own domain or completely offline](#a-without-your-own-domain-or-completely-offline)
    - [Redirecting DNS](#redirecting-dns)
      - [(Recommended) Redirect for Waydroid only (custom domain)](#recommended-redirect-for-waydroid-only-custom-domain)
      - [Redirect for Waydroid only (without domain patching)](#redirect-for-waydroid-only-without-domain-patching)
      - [Redirect for both Waydroid and your host](#redirect-for-both-waydroid-and-your-host)
    - [Installing TLS certificate](#installing-tls-certificate)
  - [B. With your own domain and TLS certificate](#b-with-your-own-domain-and-tls-certificate)
  - [Starting the game](#starting-the-game)

<!-- /TOC -->

# Client setup

## Stub APK (recommended)

This method supports both real devices and Waydroid (both `libndk` and `libhoudini` work).

The idea is to make separate APK with stubbed Java API that hosts original Unity files and does not have LIAPP protection (as it exists only in Java part).

My stub implementation is available [here](https://github.com/Assasans/konofd-client-stub/releases), or you may build it [yourself](https://github.com/Assasans/konofd-client-stub).

1. Install the stub APK on your device — `adb install jp.assasans.konofd.stub.apk`
2. Start `KonoSuba: FD (stub)`

No errors should appear as the game is already patched with correct server URL and public key.

If you want to use own server that is not publicly accessible (localhost, inside LAN or a VPN), you will need to install TLS certificate on your device.

![](https://files.catbox.moe/mq72oz.webp)

## Original APK with Waydroid

Requirements:

- Linux machine (preferably [Arch Linux](https://archlinux.org/), `x86_64` or
  `arm64-v8a`), [WSL](https://learn.microsoft.com/en-us/windows/wsl/install) is not tested
- [Waydroid](https://waydro.id)
- [Rust](https://rust-lang.org) — to build the [RSA patcher](rsa-patcher)

1. You will need a kernel with the `binder` module, see your distribution documentation for more information.
2. Initialize the Waydroid image — `sudo waydroid init -s GAPPS`
3. Start Waydroid service — `sudo systemctl start waydroid-container`
4. Install `libhoudini` (`libndk` does not pass LIAPP tests) if you are not on an ARM64 host: https://github.com/casualsnek/waydroid_script (do not select `gapps`)
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

### A. Without your own domain or completely offline

#### Redirecting DNS

##### (Recommended) Redirect for Waydroid only (custom domain)

You will have to specify `--url https://static.konosuba.local/` when patching the game.

```shell
# Use custom dnsmasq config
# This has to be done every time after updating Waydroid, as it overwrites the file.
sudo sed -i 's|LXC_DHCP_CONFILE=""|LXC_DHCP_CONFILE="/var/lib/waydroid/lxc/waydroid/dnsmasq.conf"|' /usr/lib/waydroid/data/scripts/waydroid-net.sh

# Enter the server's IP here, it must be accessible from Waydroid.
local_ip="192.168.1.102"

cat <<EOF | sudo tee -a /var/lib/waydroid/lxc/waydroid/dnsmasq.conf
# Block original domain
address=/web-prd-wonder.sesisoft.com/

addn-hosts=/var/lib/waydroid/lxc/waydroid/dnsmasq.hosts
EOF

# Redirect API hosts to our IP
cat <<EOF | sudo tee -a /var/lib/waydroid/lxc/waydroid/dnsmasq.hosts
$local_ip  static.konosuba.local.
$local_ip  api.konosuba.local.
EOF
```

##### Redirect for Waydroid only (without domain patching)

Waydroid uses `dnsmasq`, so we use it to redirect `web-prd-wonder.sesisoft.com` to our server's IP.

```shell
# Use custom dnsmasq config
# This has to be done every time after updating Waydroid, as it overwrites the file.
sudo sed -i 's|LXC_DHCP_CONFILE=""|LXC_DHCP_CONFILE="/var/lib/waydroid/lxc/waydroid/dnsmasq.conf"|' /usr/lib/waydroid/data/scripts/waydroid-net.sh

# Enter the server's IP here, it must be accessible from Waydroid.
local_ip="192.168.1.102"

# Redirect API host to our IP
cat <<EOF | sudo tee -a /var/lib/waydroid/lxc/waydroid/dnsmasq.conf
address=/web-prd-wonder.sesisoft.com/$local_ip
EOF
```

##### Redirect for both Waydroid and your host

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

#### Installing TLS certificate

```shell
# If you follow "without domain patching"
openssl req -x509 -newkey rsa:4096 -sha256 -days 3650 \
  -nodes -keyout sesisoft.com.key -out sesisoft.com.crt -subj "/CN=Axel FD Server (sesisoft.com)" \
  -addext "subjectAltName=DNS:sesisoft.com,DNS:web-prd-wonder.sesisoft.com,DNS:static-prd-wonder.sesisoft.com"

# If you follow "custom domain"
openssl req -x509 -newkey rsa:4096 -sha256 -days 3650 \
  -nodes -keyout sesisoft.com.key -out sesisoft.com.crt -subj "/CN=Axel FD Server" \
  -addext "subjectAltName=DNS:api.konosuba.local,DNS:static.konosuba.local"
openssl x509 -in sesisoft.com.crt -out sesisoft.com.pem -outform PEM

# Install certificate on Waydroid
hash=$(openssl x509 -subject_hash_old -in sesisoft.com.pem | head -1)
sudo cp sesisoft.com.pem /var/lib/waydroid/overlay/system/etc/security/cacerts/$hash.0
sudo chmod 644 /var/lib/waydroid/overlay/system/etc/security/cacerts/$hash.0
```

### B. With your own domain and TLS certificate

There are no steps clients have to do, assuming you have already configured a reverse proxy server with your TLS
certificate
that clients **trust** (e.g. from Let's Encrypt or Cloudflare) and have a domain resolvable by authoritative DNS
servers.

Clients just have to specify your domain while patching the game (e.g. `axel-rsa-patcher $(pidof com.nexon.konosuba) --url https://axel.assasans.dev/ --key ../pubkey.pem`).

### Starting the game

1. Start `KonoSuba: FD` in Waydroid and wait for the title screen to appear ("Connection Error" alert will appear — ignore it).
2. Build the [RSA key patcher](rsa-patcher) — `cargo build --release`.
3. And run it — `sudo RUST_LOG=debug ./target/release/axel-rsa-patcher --pid $(pidof com.nexon.konosuba) --url https://static.konosuba.local/ --key ../pubkey.pem`.
4. Press OK on the error alert, the game should now work.
