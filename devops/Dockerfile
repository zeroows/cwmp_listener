# Stage 1: Build the application using musl for static linking
FROM messense/rust-musl-cross:x86_64-musl AS builder

# Set the working directory
WORKDIR /app

# Copy the entire project into the container
COPY . /app

# Build the project
RUN cargo build --release

# Stage 2: Create a smaller image for running the application
FROM alpine:latest

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/cwmp-listener /app/

# Set environment variables
ENV APPLICATION__HOST=0.0.0.0
ENV APPLICATION__PORT=7547
ENV APPLICATION__LOG_LVL=debug

# Expose the port
EXPOSE 7547

# Set the entry point for the container
ENTRYPOINT ["./cwmp-listener"]
