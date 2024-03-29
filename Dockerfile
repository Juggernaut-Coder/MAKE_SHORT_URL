#Build using rust version on my local machine
FROM rust:1.77 as builder

#Create a cargo
RUN USER=root cargo new --bin myUrlShortener
WORKDIR /myUrlShortener

#Copy manifests
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock