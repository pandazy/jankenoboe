# Dockerfile for Jankenoboe e2e testing
# Builds the binary from source, installs it system-wide, then runs e2e tests
# against the installed binary — simulating real user installation.

FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y sqlite3 && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.toml ./
COPY src/ src/
COPY templates/ templates/
COPY docs/init-db.sql docs/init-db.sql

# Build release binary
RUN cargo build --release

# --- Runtime stage ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y sqlite3 jq && rm -rf /var/lib/apt/lists/*

# Install binary to /usr/local/bin (simulates user installation)
COPY --from=builder /build/target/release/jankenoboe /usr/local/bin/jankenoboe
RUN chmod +x /usr/local/bin/jankenoboe

# Copy schema and e2e test scripts
COPY docs/init-db.sql /app/init-db.sql
COPY e2e/run_tests.sh /app/run_tests.sh
RUN chmod +x /app/run_tests.sh

WORKDIR /app

# Initialize the database
RUN mkdir -p /app/db && sqlite3 /app/db/datasource.db < /app/init-db.sql

ENV JANKENOBOE_DB=/app/db/datasource.db

CMD ["/app/run_tests.sh"]