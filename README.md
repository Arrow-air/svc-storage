![Arrow Banner](https://github.com/Arrow-air/.github/raw/main/profile/assets/arrow_v2_twitter-banner_neu.png)

# svc-storage Service

![Rust
Checks](https://github.com/arrow-air/svc-storage/actions/workflows/rust_ci.yml/badge.svg?branch=main)
![Python Flake8](https://github.com/arrow-air/svc-storage/actions/workflows/python_ci.yml/badge.svg?branch=main)
![Arrow DAO
Discord](https://img.shields.io/discord/853833144037277726?style=plastic)

## :telescope: Overview
svc-storage is responsible for storing and retrieving data from the Arrow database and other storage.
It's meant to be used only by other internal services via GRPC interface.
- server - [bin] target to run gRPC server
- client - [lib] target for other services to import and use

Directory:
- `server/src/`: Source Code and Unit Tests of the server
- `client-grpc/src/`: Source Code and Unit Tests of the GRPC client
- `tests/`: Integration Tests
- `docs/`: Module Documentation

## Installation

Install Rust with [Rustup](https://www.rust-lang.org/tools/install).

```bash
# After cloning the repository
python3 -m pip install -r requirements.txt

# Adds custom pre-commit hooks to .git through cargo-husky dependency
# !! Required for developers !!
cargo test
```

## :scroll: Documentation
The following documents are relevant to this service:
- [Concept of Operations](./docs/conops.md)
- [Requirements & User Stories](./docs/requirements.md)
- [ICD](./docs/icd.md)
- [SDD](./docs/sdd.md)

## :busts_in_silhouette: Arrow DAO
Learn more about us:
- [Website](https://www.arrowair.com/)
- [Arrow Docs](https://www.arrowair.com/docs/intro)
- [Discord](https://discord.com/invite/arrow)

## :exclamation: Treatment of `Cargo.lock`
If you are building a non-end product like a library, include `Cargo.lock` in `.gitignore`.

If you are building an end product like a command line tool, check `Cargo.lock` to the git.

Read more about it [here](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html);

## Cockroachdb
The database connection requires TLS.
When running cockroachdb for development, certificates will automatically be generated and used by the server.

### Cockroachdb key generation
Certificates are automatically generated by the `cockroachdb-init` container in case they're missing.
They are written to a dedicated `cockroachdb-ssl` volume so they can be shared with the services that need them.
Note that these are all snake-oil and should not be used for production.

TODO: Get rid of user/pass and use client certificates instead.

**Set up your environment:**
Create an .env file with the following contents (The password should be the password you will use for the svc_storage user):
```
SOURCE_PATH=.
DOCKER_NAME=arrow-svc-storage
PACKAGE_NAME=svc-storage
PUBLISH_PACKAGE_NAME=svc-storage-client-grpc

# Used by the server to connect to the CockroachDB backend
PG__USER=svc_storage
PG__DBNAME=arrow
PG__HOST=cockroachdb
PG__PORT=26257
PG__SSLMODE=require
DB_CA_CERT=/cockroach/ssl/certs/ca.crt
DB_CLIENT_CERT=/cockroach/ssl/certs/client.svc_storage.crt
DB_CLIENT_KEY=/cockroach/ssl/certs/client.svc_storage.key.pk8

# Used by the server and docker compose to map the right ports
DOCKER_PORT_REST=8000
DOCKER_PORT_GRPC=50051
HOST_PORT_REST=8003
HOST_PORT_GRPC=50003

# Used for the grpc client to know which port to connect to
SERVER_PORT_GRPC=50003
```
Run the example:
```
make rust-example-grpc
```


### MacOS troubleshooting

1. Currently, the `make build` command does not work on MacOS with M1 processor so running the example will fail.
As a workaround we can start only the cockroachdb with `docker-compose up cockroachdb` and then `cargo run` to start the svc-storage server.

2. The other problem on MacOS is that it doesn't trust automatically generated cockroachdb certificates. (Errors like "certificate not trusted" or "bad certificate")
As a workaround we need to add/change these env variables in .env file
```
PG__HOST=localhost
PG__SSLMODE=disable
USE_TLS=false
```
and then modify the `docker-compose.yml` file and replace line
`command: start-single-node --certs-dir=/cockroach/ssl/certs --advertise-addr=cockroachdb`
with:
`command: start-single-node --insecure --advertise-addr=cockroachdb`

This will run the cockroachdb and svc-storage without TLS using username/password.
