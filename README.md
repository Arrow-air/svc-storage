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
- `client/src/`: Source Code and Unit Tests of the client
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
The database connection requires TLS. When running cockroachdb in development, you will need to generate certificates which can be used by the server.

### Cockroachdb key generation
TODO: automate this so developers don't need to think about it. Can be a make target to check if the certs and keys dir exist and creates them if not present.

There's an open issue with SSL in our docker image, so it's currently not possible to run the example client with docker. If you want to run it manually, you can follow these steps:

**Create database certs:**
```
mkdir certs
mkdir keys
docker run --rm -v $(pwd)/certs:/cockroach/certs -v $(pwd)/keys:/cockroach/keys -it cockroachdb/cockroach:v22.1.9 cert create-ca --certs-dir=/cockroach/certs --ca-key=/cockroach/keys/ca.key
docker run --rm -v $(pwd)/certs:/cockroach/certs -v $(pwd)/keys:/cockroach/keys -it cockroachdb/cockroach:v22.1.9 cert create-client root --certs-dir=/cockroach/certs --ca-key=/cockroach/keys/ca.key
docker run --rm -v $(pwd)/certs:/cockroach/certs -v $(pwd)/keys:/cockroach/keys -it cockroachdb/cockroach:v22.1.9 cert create-node localhost $(hostname) --certs-dir=/cockroach/certs --ca-key=/cockroach/keys/ca.key
sudo cp certs/ca.crt /etc/ca-certificates/cockroachdb-ca.crt
```
**Set up your environment:**
Create an .env file with the following contents (The password should be the password you will use for the svc_storage user):
```
PG__USER=svc_storage
PG__DBNAME=arrow
PG__PASSWORD="<your pass>"
PG__HOST=localhost
PG__PORT=26257
PG__SSLMODE=require
DB_CA_CERT=/etc/ca-certificates/cockroachdb-ca.crt
SOURCE_PATH=.

DOCKER_NAME=arrow-svc-storage
PACKAGE_NAME=svc-storage

PUBLISH_PACKAGE_NAME=svc-storage-client-grpc
DOCKER_PORT_REST=8000
DOCKER_PORT_GRPC=50051
HOST_PORT_REST=8003
HOST_PORT_GRPC=50003
```
**Start CockroachDB and create your database + user:**
```
source .env
docker compose up -d cockroachdb
docker exec -it arrow-svc-storage-example-cockroachdb sh
cockroach sql --certs-dir /cockroach/certs/
CREATE DATABASE arrow;
CREATE USER svc_storage WITH PASSWORD '<your password>';
GRANT ALL PRIVILEGES ON DATABASE arrow TO svc_storage;
```

Run the example:
```
cargo run &
cargo run --example grpc
```
