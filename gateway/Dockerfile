FROM rust:buster as build

COPY ./gateway ./gateway
COPY ./requester ./requester
COPY ./structs ./structs
COPY ./jwtchecker ./jwtchecker
WORKDIR ./gateway

RUN cargo build --release

FROM debian:buster
COPY --from=build /gateway/target/release/gateway .
RUN apt-get update && apt-get install -y libssl1.1 && apt clean && rm -rf /var/lib/apt/lists/*
CMD ["./gateway"]
