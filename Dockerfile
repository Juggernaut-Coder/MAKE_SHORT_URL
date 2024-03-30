#Build using rust version on my local machine
FROM rust:1.77 as build

#Create select shell project, workdir and copy manifest 
RUN USER=root cargo new --bin myUrlShortener
WORKDIR /myUrlShortener
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN rm src/*.rs
COPY ./src ./src
RUN cargo build --release

FROM rust:1.77
COPY --from=build /myUrlShortener/target/release/myUrlShortener /usr/local/bin/



CMD ["myUrlShortener"]




