## Multi-stage build for rust-sixtysix example HTTP server
## Usage:
##   docker build -t rust-sixtysix:dev .
##   docker run -p 8080:8080 rust-sixtysix:dev

FROM rust:1.93 AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock* ./
# Create dummy main to cache deps
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    mkdir examples && echo "fn main() {}" > examples/server.rs && \
    cargo build --release --example server 2>/dev/null || true
COPY . .
# Touch files to ensure rebuild
RUN touch src/lib.rs examples/server.rs && \
    cargo build --release --example server

FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app
COPY --from=build /src/target/release/examples/server /app/server
EXPOSE 8080
USER nonroot:nonroot
ENV PORT=8080
ENTRYPOINT ["/app/server"]
