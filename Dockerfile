# Build stage
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/rust-game ./game
COPY ./static ./static
EXPOSE 8080
CMD ["./game"]
