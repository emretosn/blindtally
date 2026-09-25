# blindtally

A toy electronic voting system where the server counts votes **without ever
being able to read them**. Ballots are encrypted with fully homomorphic
encryption (FHE) using [TFHE-rs](https://github.com/zama-ai/tfhe-rs), so the
server adds up ciphertexts and only the key holder can decrypt the totals.

This is a learning project for Rust, TFHE-rs, and HTTP services in Rust
(axum on the server, reqwest on the client). It is not meant for real
elections.

## How it works

TFHE uses two keys:

- The **client key** is secret. It encrypts ballots and decrypts results, and
  it never leaves the client's machine.
- The **server key** is public. It lets the server compute on ciphertexts
  but not decrypt them.

The election runs in five steps:

1. **keygen:** the client generates a client key and saves it to `client_key.bin`.
2. **open:** the client derives a compressed server key (~68 MB, compared
   with ~190 MB uncompressed) and uploads it to the server.
3. **vote:** each vote is a candidate ID (`0` = Alice, `1` = Bob) encrypted
   as an `FheUint8` and posted to the server.
4. **tally:** for every ballot and every candidate, the server computes
   `encrypted_choice == candidate_id` homomorphically and adds the encrypted
   result (1 or 0) to that candidate's encrypted `FheUint32` count. Counts
   start from a *trivial* encryption of zero, which the server can create
   with its key alone.
5. **result:** the client downloads the encrypted counts and decrypts them
   with the client key.

At no point can the server see how any individual ballot was cast, or what
the totals are.

## Project layout

A Cargo workspace with three crates:

| Crate | Path | Role |
|---|---|---|
| `blindtally-core` | `core/` | Shared types (`Candidate`, `Ballot`), the `AppError` enum, and bincode helpers |
| `blindtally-server` | `server/` | axum HTTP server that stores ballots in memory and runs the encrypted tally |
| `blindtally-client` | `client/` | CLI that holds the client key, encrypts votes, and decrypts results |

```
core/src/
  election.rs   candidates and the encrypted Ballot type
  errors.rs     AppError (thiserror)
  io.rs         bincode to/from bytes and files
server/src/
  main.rs       CLI args, tracing, starts the server
  routes.rs     shared state and HTTP handlers
  error.rs      ApiError: maps errors to HTTP status codes
  tally.rs      the homomorphic counting
client/src/
  main.rs       CLI subcommands
  api.rs        typed wrapper around the server's HTTP API
  keys.rs       key generation and client key storage
  error.rs      ClientError
```

## Running an election

Requires a recent stable Rust toolchain (edition 2024). Always use
`--release`: TFHE is far slower in debug builds.

Start the server in one terminal:

```sh
cargo run --release -p blindtally-server
# listens on 127.0.0.1:3000; change with --addr 0.0.0.0:8080
```

Then use the client in another:

```sh
cargo run --release -p blindtally-client -- keygen
cargo run --release -p blindtally-client -- open
cargo run --release -p blindtally-client -- vote alice
cargo run --release -p blindtally-client -- vote bob
cargo run --release -p blindtally-client -- vote alice
cargo run --release -p blindtally-client -- tally
cargo run --release -p blindtally-client -- result
```

```
Alice: 2 votes
Bob: 1 votes
```

Every client command accepts `--server <url>` (default
`http://127.0.0.1:3000`). `client_key.bin` is read from the current
directory, so run all client commands from the same place.

Tallying took about a second per ballot on a laptop, so expect `tally` to
take a while with many votes.

## HTTP API

All request and response bodies are [bincode](https://github.com/bincode-org/bincode)-encoded.
Errors come back as plain text.

| Method | Path | Body | Success | Errors |
|---|---|---|---|---|
| `POST` | `/keys` | `CompressedServerKey` | `201` | `400` bad body, `409` key already set |
| `POST` | `/ballots` | `Ballot` | `201` | `400` bad body, `409` election not open |
| `POST` | `/tally` | none | `200`, text such as `tallied 3 ballots` | `409` election not open |
| `GET` | `/results` | none | `200`, `Vec<FheUint32>` (one count per candidate) | `404` no tally yet |

## Limitations

- **State is in memory only.** Restarting the server loses the election.
- **No authentication.** Anyone who can reach the server can vote, and can
  vote more than once.
- **Results are not cleared by new votes.** `/results` returns the latest
  tally until `tally` runs again.
- **Ballots are not validated.** A ballot encrypting a value other than a
  known candidate ID is counted for no one.
- **Plain HTTP only.** reqwest is built without TLS, so `https://` URLs
  won't work.
- **Two hardcoded candidates.** Alice and Bob are set in `core/src/election.rs`.
