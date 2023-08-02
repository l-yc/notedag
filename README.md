# notedag

wip experimental tree-like jupyter frontend built with rust and svelte

## build

must build server after frontend because static files are embedded into the emitted binary

```sh
cd notedag-frontend
npm run build
cd ..

cd notedag-server
cargo build --release
cd ..

ROOT=./tests/ ./notedag-server/target/release/notedag-server
```

to build the binary statically

```sh
PKG_CONFIG_ALLOW_CROSS=1 cargo build --release --target x86_64-unknown-linux-musl
```

note that you will need to install zeromq 

## dev

### notedag-frontend

use default API host (`http://127.0.0.1:8080`)

```sh
npm run dev -- --open
```

env vars:
- `PUBLIC_API_URL`:

custom example:

```sh
PUBLIC_API_URL=http://127.0.0.1:1337 npm run dev -- --open
```

### notedag-server

quickstart:

```sh
ROOT=../tests/ RUST_LOG=info cargo run
```

env vars:
- `PORT` (default `8080`): port for api server
- `ROOT` (default ` `): root dir of file tree

example:

```sh
PORT=1337 ROOT=../tests/ RUST_LOG=info cargo run
```
