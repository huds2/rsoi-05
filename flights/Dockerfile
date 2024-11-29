FROM rust:buster as build

COPY ./flights ./flights
COPY ./requester ./requester
COPY ./structs ./structs
COPY ./jwtchecker ./jwtchecker
WORKDIR ./flights

RUN cargo build --release

FROM debian:buster
COPY --from=build /flights/target/release/flights .
CMD ["./flights"]
