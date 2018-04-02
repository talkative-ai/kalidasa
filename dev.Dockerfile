FROM alpine:latest

WORKDIR /main/

COPY target/x86_64-unknown-linux-musl/debug/kalidasa /usr/local/bin

ENTRYPOINT [ "kalidasa" ]

EXPOSE 8080