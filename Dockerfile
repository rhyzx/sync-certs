FROM clux/muslrust:stable AS build

WORKDIR /repo

RUN cargo init
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY src ./src
RUN touch ./src/main.rs # fix build cache
RUN cargo build --release

##############
FROM scratch
COPY --from=build /repo/target/x86_64-unknown-linux-musl/release/sync-certs /sync-certs
CMD ["/sync-certs"]
