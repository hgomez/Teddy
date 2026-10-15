<div align="center">
  <img src="TeddyBears4Ever.jpg" alt="Teddy logo" width="200"/>
  <h1>Teddy</h1>
  <p><em>The smart bear's agent</em></p>
</div>

---

Teddy is a small agent daemon written in Rust ([Actix-web 4](https://actix.rs)) that can execute commands and upload/download files on a machine. Every operation is exposed through a simple REST API.

It is deployable on Linux, macOS and Windows.

## Features

- **Remote execution** — run a single command (no shell, no multi-args) and get back its status, stdout and stderr as JSON
- **File download** — stream any file from the machine
- **File upload** — push files via multipart form (up to 100 MB)
- **Prometheus metrics** — exposed on a dedicated port (`:9000/metrics`)
- **Self-contained** — a single binary, low memory footprint, configuration optional

## API

| Method | Path              | Auth required | Description                                    |
|--------|-------------------|:-------------:|------------------------------------------------|
| `GET`  | `/`               | No            | Welcome message                                 |
| `GET`  | `/ping`           | No            | Health check (returns `pong`)                   |
| `POST` | `/admin/exec`     | Yes           | Execute a command, returns `{status,stdout,stderr}` |
| `POST` | `/admin/download` | Yes           | Download a file (`?filename=<path>` query param) |
| `POST` | `/admin/upload`   | Yes           | Upload a file (multipart form, field `uploaded_file`) |

### Authentication

All `/admin/*` endpoints are guarded: requests must carry an `Authorization` header equal to `Base64(user:password)`.

```sh
curl -H "Authorization: $(printf 'teddy:rocks' | base64)" \
     -X POST http://localhost:3000/admin/exec \
     -H "Content-Type: application/json" \
     -d '{"command": "uname", "parameters": "-a"}'
```

## Configuration

Configuration is optional. If no `config.json` is found next to the binary (or if it fails to parse), defaults are used.

```json
{
  "host": "0.0.0.0",
  "port": 3000,
  "user": "teddy",
  "password": "rocks"
}
```

| Key        | Default    | Description                          |
|------------|------------|--------------------------------------|
| `host`     | `0.0.0.0`  | Address the API binds to             |
| `port`     | `3000`     | Port of the REST API                 |
| `user`     | `teddy`    | Username for `/admin/*` endpoints    |
| `password` | `rocks`    | Password for `/admin/*` endpoints    |

Prometheus metrics are always exposed on **`:9000/metrics`**, regardless of the API port.

## Build

Requires a [Rust](https://rustup.rs) toolchain (edition 2024).

```sh
cargo build
```

## Run

```sh
cargo run
```

The REST API listens on `:3000` (or whatever is configured), and Prometheus metrics on `:9000/metrics`.

## Test

```sh
cargo test
```

## Project layout

```
src/
├── main.rs        # bootstrap and route table (/admin/* guarded)
├── conf.rs        # configuration loading (config.json, defaults in code)
├── guards.rs      # auth guard: Authorization = Base64(user:password)
└── handlers/      # one module per endpoint
    ├── welcome.rs
    ├── ping.rs
    ├── download.rs
    ├── upload.rs
    └── execute.rs
```

## Validation gate

CI (`.github/workflows/ci.yml`) enforces the following checks on every commit — run them locally before pushing:

```sh
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test
cargo audit
```

## Security notice

Teddy is a **remote-execution agent by design**: `/admin/exec` and `/admin/download` allow arbitrary commands and file reads. Never expose it on an untrusted network, always protect it with strong credentials, and never log or leak the `Authorization` token.
