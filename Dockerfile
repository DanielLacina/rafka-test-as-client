FROM rust:1.80.1
COPY . .
RUN cargo build --release
CMD ["./target/release/rafka-test-as-client"]
