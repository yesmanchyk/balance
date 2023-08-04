FROM rust:1.71
WORKDIR /usr/src/actix-rest
COPY . .
RUN cargo install --path .
EXPOSE 8080
CMD ["/usr/src/actix-rest/target/release/actix-rest"]