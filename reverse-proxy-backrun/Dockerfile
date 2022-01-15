
# Start with a rust alpine image
FROM rust:1.57-alpine
# This is important, see https://github.com/rust-lang/docker-rust/issues/85
ENV RUSTFLAGS="-C target-feature=-crt-static"
# if needed, add additional dependencies here
RUN apk add --no-cache musl-dev
# set the workdir and copy the source into it
WORKDIR /app
COPY ./Cargo.* /app/

COPY ./src /app/src
COPY ./config /app/config
# do a release build
RUN cargo build --release
RUN strip target/release/reverse-proxy-backrun

# use a plain alpine image, the alpine version needs to match the builder
FROM alpine:3.14
# if needed, install additional dependencies here
RUN apk add --no-cache libgcc
WORKDIR /app
# copy the binary into the final image
COPY --from=0 /app/target/release/reverse-proxy-backrun .
COPY --from=0 /app/config ./config

# Add Dockerfile for documentation
COPY ./Dockerfile .
# set the binary as entrypoint
ENTRYPOINT ["./reverse-proxy-backrun"]