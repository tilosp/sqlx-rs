<h1 align="center">SQLx</h1>
<div align="center">
 <strong>
   🧰 The Rust SQL Toolkit
 </strong>
</div>

<br />

<div align="center">
  <!-- Github Actions -->
  <img src="https://img.shields.io/github/workflow/status/launchbadge/sqlx/SQLx?style=flat-square" alt="actions status" />
  <!-- Version -->
  <a href="https://crates.io/crates/sqlx">
    <img src="https://img.shields.io/crates/v/sqlx.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Discord -->
  <a href="https://discord.gg/uuruzJ7">
    <img src="https://img.shields.io/discord/665528275556106240?style=flat-square" alt="chat" />
  </a>
  <!-- Docs -->
  <a href="https://docs.rs/sqlx">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/sqlx">
    <img src="https://img.shields.io/crates/d/sqlx.svg?style=flat-square"
      alt="Download" />
  </a>
</div>

<div align="center">
  <h4>
    <a href="#install">
      Install
    </a>
    <span> | </span>
    <a href="#usage">
      Usage
    </a>
    <span> | </span>
    <a href="https://docs.rs/sqlx">
      Docs
    </a>
  </h4>
</div>

<br />

<div align="center">
  <sub>Built with ❤️ by <a href="https://launchbadge.com">The LaunchBadge team</a></sub>
</div>

<br />

SQLx is an async, pure Rust<sub>†</sub> SQL crate featuring compile-time checked queries without a DSL.

 * **Truly Asynchronous**. Built from the ground-up using async/await for maximum concurrency.

 * **Type-safe SQL** (if you want it) without DSLs. Use the `query!()` macro to check your SQL and bind parameters at
 compile time. (You can still use dynamic SQL queries if you like.)

 * **Database Agnostic**. Support for [PostgreSQL], [MySQL], and [SQLite].

 * **Pure Rust**. The Postgres and MySQL/MariaDB drivers are written in pure Rust using **zero** unsafe<sub>††</sub> code.

 * **Runtime Agnostic**. Works on [async-std](https://crates.io/crates/async-std) or [tokio](https://crates.io/crates/tokio) with the `runtime-async-std-native-tls` or `runtime-tokio-native-tls` cargo feature flag.

<sub><sup>† The SQLite driver uses the libsqlite3 C library as SQLite is an embedded database (the only way
we could be pure Rust for SQLite is by porting _all_ of SQLite to Rust).</sup></sub>

<sub><sup>†† SQLx uses `#![forbid(unsafe_code)]` unless the `sqlite` feature is enabled. As the SQLite driver interacts
with C, those interactions are `unsafe`.</sup></sub>

[PostgreSQL]: http://postgresql.org/
[SQLite]: https://sqlite.org/
[MySQL]: https://www.mysql.com/

---

 * Cross-platform. Being native Rust, SQLx will compile anywhere Rust is supported.

 * Built-in connection pooling with `sqlx::Pool`.

 * Row streaming. Data is read asynchronously from the database and decoded on-demand.

 * Automatic statement preparation and caching. When using the high-level query API (`sqlx::query`), statements are
   prepared and cached per-connection.

 * Simple (unprepared) query execution including fetching results into the same `Row` types used by
   the high-level API. Supports batch execution and returning results from all statements.

 * Transport Layer Security (TLS) where supported ([MySQL] and [PostgreSQL]).

 * Asynchronous notifications using `LISTEN` and `NOTIFY` for [PostgreSQL].

 * Nested transactions with support for save points.

## Install

SQLx is compatible with the [`async-std`] and [`tokio`] runtimes.

[`async-std`]: https://github.com/async-rs/async-std
[`tokio`]: https://github.com/tokio-rs/tokio

**async-std**

```toml
# Cargo.toml
[dependencies]
sqlx = "0.3"
```

**tokio**

```toml
# Cargo.toml
[dependencies]
sqlx = { version = "0.3", default-features = false, features = [ "runtime-tokio-native-tls", "macros" ] }
```

#### Cargo Feature Flags

 * `runtime-async-std-native-tls` (on by default): Use the `async-std` runtime.

 * `runtime-tokio-native-tls`: Use the `tokio` runtime. Mutually exclusive with the `runtime-async-std-native-tls` feature.

 * `postgres`: Add support for the Postgres database server.

 * `mysql`: Add support for the MySQL (and MariaDB) database server.

 * `sqlite`: Add support for the self-contained [SQLite](https://sqlite.org/) database engine.

 * `uuid`: Add support for UUID (in Postgres).

 * `chrono`: Add support for date and time types from `chrono`.

 * `time`: Add support for date and time types from `time` crate (alternative to `chrono`, prefered by `query!` macro, if both enabled)

 * `bigdecimal`: Add support for `NUMERIC` using the `bigdecimal` crate.

 * `ipnetwork`: Add support for `INET` and `CIDR` (in postgres) using the `ipnetwork` crate.

 * `json`: Add support for `JSON` and `JSONB` (in postgres) using the `serde_json` crate.

 * `tls`: Add support for TLS connections.

## Usage

### Quickstart

```rust
use std::env;

use sqlx::postgres::PgPool;
// use sqlx::mysql::MySqlPool;
// etc.

#[async_std::main] // or #[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // Create a connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL")?).await?;

    // Make a simple query to return the given parameter
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool).await?;

    assert_eq!(row.0, 150);

    Ok(())
}
```

### Connecting

A single connection can be established using any of the database connection types and calling `connect()`.

```rust
use sqlx::Connect;

let conn = SqliteConnection::connect("sqlite::memory:").await?;
```

Generally, you will want to instead create a connection pool (`sqlx::Pool`) in order for your application to
regulate how many server-side connections it's using.

```rust
let pool = MySqlPool::new("mysql://user:pass@host/database").await?;
```

### Querying

In SQL, queries can be separated into prepared (parameterized) or unprepared (simple). Prepared queries have their
query plan _cached_, use a binary mode of communication (lower bandwidth and faster decoding), and utilize parameters
to avoid SQL injection. Unprepared queries are simple and intended only for use case where a prepared statement
will not work, such as various database commands (e.g., `PRAGMA` or `SET` or `BEGIN`).

SQLx supports all operations with both types of queries. In SQLx, a `&str` is treated as an unprepared query
and a `Query` or `QueryAs` struct is treated as a prepared query.

```rust
// low-level, Executor trait
conn.execute("BEGIN").await?; // unprepared, simple query
conn.execute(sqlx::query("DELETE FROM table")).await?; // prepared, cached query
```

We should prefer to use the high level, `query` interface whenever possible. To make this easier, there are finalizers
on the type to avoid the need to wrap with an executor.

```rust
sqlx::query("DELETE FROM table").execute(&mut conn).await?;
sqlx::query("DELETE FROM table").execute(&pool).await?;
```

The `execute` query finalizer returns the number of affected rows, if any, and drops all received results.
In addition, there are `fetch`, `fetch_one`, `fetch_optional`, `fetch_all`, and `fetch_scalar` to receive results.

The `Query` type returned from `sqlx::query` will return `Row<'conn>` from the database. Column values can be accessed
by ordinal or by name with `row.get()`. As the `Row` retains an immutable borrow on the connection, only one
`Row` may exist at a time.

The `fetch` query finalizer returns a stream-like type that iterates through the rows in the result sets.

```rust
let mut cursor = sqlx::query("SELECT * FROM users WHERE email = ?")
    .bind(email)
    .fetch(&mut conn);

while let Some(row) = cursor.next().await? {
    // map the row into a user-defined domain type
}
```

To assist with mapping the row into a domain type, there are two idioms that may be used:

```rust
let mut stream = sqlx::query("SELECT * FROM users")
    .map(|row: PgRow| {
        // map the row into a user-defined domain type
    })
    .fetch(&mut conn);
```

```rust
#[derive(sqlx::FromRow)]
struct User { name: String, id: i64 }

let mut stream = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ? OR name = ?")
    .bind(user_email)
    .bind(user_name)
    .fetch(&mut conn);
```

Instead of a stream of results, we can use `fetch_one` or `fetch_optional` to request one required or optional result
from the database.

### Compile-time verification

We can use the macro, `sqlx::query!` to achieve compile-time syntactic and semantic verification of the SQL, with
an output to an anonymous record type where each SQL column is a Rust field (using raw identifiers where needed).

```rust
let countries = sqlx::query!(
        "
SELECT country, COUNT(*) as count
FROM users
GROUP BY country
WHERE organization = ?
        ",
        organization
    )
    .fetch_all(&pool) // -> Vec<{ country: String, count: i64 }>
    .await?;

// countries[0].country
// countries[0].count
```

Differences from `query()`:

 * The input (or bind) parameters must be given all at once (and they are compile-time validated to be
   the right number and the right type).

 * The output type is an anonymous record. In the above example the type would be similar to:

    ```rust
    { country: String, count: i64 }
    ```

 * The `DATABASE_URL` environment variable must be set at build time to a database which it can prepare
   queries against; the database does not have to contain any data but must be the same
   kind (MySQL, Postgres, etc.) and have the same schema as the database you will be connecting to at runtime.

   For convenience, you can use a .env file to set DATABASE_URL so that you don't have to pass it every time:

   ```
   DATABASE_URL=mysql://localhost/my_database
   ```

The biggest downside to `query!()` is that the output type cannot be named (due to Rust not
officially supporting anonymous records). To address that, there is a `query_as!()` macro that is identical
except that you can name the output type.


```rust
// no traits are needed
struct Country { country: String, count: i64 }

let countries = sqlx::query_as!(Country,
        "
SELECT country, COUNT(*) as count
FROM users
GROUP BY country
WHERE organization = ?
        ",
        organization
    )
    .fetch_all() // -> Vec<Country>
    .await?;

// countries[0].country
// countries[0].count
```

## Safety

This crate uses `#![forbid(unsafe_code)]` to ensure everything is implemented in 100% Safe Rust.

If the `sqlite` feature is enabled, this is downgraded to `#![deny(unsafe_code)]` with `#![allow(unsafe_code)]` on the
`sqlx::sqlite` module. There are several places where we interact with the C SQLite API. We try to document each call for the invariants we're assuming. We absolutely welcome auditing of, and feedback on, our unsafe code usage.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
