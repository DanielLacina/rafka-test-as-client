FROM rust:1.80.1
RUN apt-get update && apt-get install -y protobuf-compiler
COPY . .
RUN cargo build --release
CMD ["./target/release/broker"]
