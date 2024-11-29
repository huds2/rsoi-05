FROM rust:buster as build

COPY ./bonuses ./bonuses
COPY ./requester ./requester
COPY ./structs ./structs
COPY ./jwtchecker ./jwtchecker
WORKDIR ./bonuses

RUN cargo build --release

FROM debian:buster
COPY --from=build /bonuses/target/release/bonuses .
CMD ["./bonuses"]
