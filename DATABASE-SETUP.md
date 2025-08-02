<!-- TOC depthfrom:2 -->

- [Setting up mTLS for PostgreSQL](#setting-up-mtls-for-postgresql)
  - [PostgreSQL configuration](#postgresql-configuration)
  - [Axel configuration](#axel-configuration)
- [Running migrations](#running-migrations)

<!-- /TOC -->

# Database setup

Create a PostgreSQL user and a database for Axel:

```sh
sudo -u postgres createuser axel
sudo -u postgres createdb axel --owner axel
```

Edit your configuration file and set your connection settings:

```toml
[database.pool]
host = "10.66.66.1"
port = 5432
user = "axel"
dbname = "axel"
```

## Setting up mTLS for PostgreSQL

Although it is not required, I highly recommend using Mutual TLS authentication for PostgreSQL connections instead of password authentication if you are running a production server.
This also has the benefit of having an encrypted database communication.

### PostgreSQL configuration

Client will verify host against its Subject Alternative Name attributes, or against its Common Name if no Subject Alternative Name is present.
More information can be found at https://www.postgresql.org/docs/current/libpq-ssl.html.

Change certificate Common Name attributes as needed.
Change `DNS:aqua.assasans.dev,IP:10.66.66.1` to your own domain or IP address.

```sh
# Generate root CA
openssl req -x509 -nodes -newkey ec \
  -pkeyopt ec_paramgen_curve:prime256v1 \
  -pkeyopt ec_param_enc:named_curve \
  -sha384 -days 3650 \
  -keyout ca.key -out ca.crt \
  -subj "/CN=aqua.assasans.dev PostgreSQL root CA"

# Generate server certificate request
openssl req -new -nodes -newkey ec \
  -pkeyopt ec_paramgen_curve:prime256v1 \
  -sha384 \
  -keyout server.key -out server.csr \
  -subj "/CN=aqua.assasans.dev PostgreSQL cluster" \
  -addext "subjectAltName=DNS:aqua.assasans.dev,IP:10.66.66.1"

# ...and sign it with the root CA
openssl x509 -req -in server.csr -days 365 \
  -CA ca.crt -CAkey ca.key -CAcreateserial \
  -sha384 \
  -copy_extensions copy \
  -out server.crt
```

```sh
# Copy TLS files to the PostgreSQL data directory
sudo mkdir /var/lib/postgres/data/tls
sudo cp -v ca.crt server.crt server.key /var/lib/postgres/data/tls
sudo chown -Rv postgres:postgres /var/lib/postgres/data/tls
sudo chmod 600 /var/lib/postgres/data/tls/server.key

# Configure PostgreSQL to use TLS
echo <<EOF
# TLS / SSL config
ssl = on
ssl_cert_file = '/var/lib/postgres/data/tls/server.crt'
ssl_key_file = '/var/lib/postgres/data/tls/server.key'
ssl_ca_file = '/var/lib/postgres/data/tls/ca.crt'
ssl_min_protocol_version = 'TLSv1.3'
ssl_max_protocol_version = 'TLSv1.3'
EOF | sudo tee -a /var/lib/postgres/data/postgresql.conf

# Configure host-based authentication
echo <<EOF
# Require mTLS for all non-local connections
hostssl all all 0.0.0.0/0 cert clientcert=verify-full
hostssl all all ::0/0 cert clientcert=verify-full

# Reject non-TLS non-local connections
hostnossl all all 0.0.0.0/0 reject
hostnossl all all ::0/0 reject
EOF | sudo tee -a /var/lib/postgres/data/pg_hba.conf
```

### Axel configuration

Common Name of the client certificate must match the database user name.
More information can be found at https://www.postgresql.org/docs/current/auth-cert.html.

```sh
# Generate client certificate request
openssl req -new -nodes -newkey ec \              
  -pkeyopt ec_paramgen_curve:prime256v1 \                               
  -sha384 \              
  -keyout axel.key -out axel.csr \
  -subj "/CN=axel"

# ...and sign it with the root CA
openssl x509 -req -in axel.csr -days 365 \      
  -CA ca.crt -CAkey ca.key -CAcreateserial \
  -sha384 -out axel.crt
```

Edit your configuration file to include the database TLS files, matching the paths where you saved them:

```toml
[database.tls]
ca-cert = "/home/assasans/dev/00-aqua/ca.crt"
client-cert = "config/axel.crt"
client-key = "config/axel.key"
```

## Running migrations

You need to initialize the database before you can run the server.
Migrations create the necessary tables and initial data.

Edit the command to match your database connection settings.

```sh
./migrations/migrate.sh "host=10.66.66.1 \
  port=5432 \
  dbname=axel \
  user=axel \
  sslrootcert=/home/assasans/dev/00-aqua/ca.crt \
  sslcert=config/axel.crt \
  sslkey=config/axel.key \
  sslmode=verify-full"
```
