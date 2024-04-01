# Build stage with Rust environment
FROM rust:1.77 as build

RUN cargo new --bin myUrlShortener
WORKDIR /myUrlShortener
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN rm src/*.rs
COPY ./src ./src
RUN cargo build --release

# Check the contents of the target directory
RUN ls -l /myUrlShortener/target/release/myUrlShortener || { echo "myUrlShortener not found in /myUrlShortener/target/release/"; exit 1; }

# Final stage with Debian slim
FROM rust:1.77-slim as runtime

# Install necessary system dependencies
RUN apt-get update && apt-get install -y redis-tools ca-certificates && rm -rf /var/lib/apt/lists/*

# Create app directory and copy static files
RUN mkdir -p /App/static
COPY ./static /App/static

# Copy the binary from the build stage
COPY --from=build /myUrlShortener/target/release/myUrlShortener /App/

# Set the working directory
WORKDIR /App

# Ensure the binary is executable
RUN chmod +x myUrlShortener

# Command to run the binary
CMD ["./myUrlShortener"]