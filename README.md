# jankenoboe

Web backend services for memorizing Anime songs

See also [This frontend code](https://github.com/pandazy/jankenamq-web) that uses this backend

## Development

1. Install Rust, see [here](https://www.rust-lang.org/learn/get-started)

2. Run the server

```bash
cargo run
```

Alternatively, you can specify the SQLite database path and the port number.

For example:

```bash
DB_PATH="/My/Path/To/datasource.db" cargo run
```

```bash
PORT=3001 DB_PATH="/My/Path/To/datasource.db" cargo run
```
