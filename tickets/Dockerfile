FROM rust:buster as build

COPY ./tickets ./tickets
COPY ./requester ./requester
COPY ./structs ./structs
COPY ./jwtchecker ./jwtchecker
WORKDIR ./tickets

RUN cargo build --release

FROM debian:buster
COPY --from=build /tickets/target/release/tickets .
CMD ["./tickets"]
