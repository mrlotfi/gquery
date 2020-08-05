FROM rust:alpine AS builder

RUN apk add  musl-dev
WORKDIR /usr/src/gquery
COPY . .
RUN cargo install --path .
RUN ls

FROM scratch
COPY --from=builder /usr/local/cargo/bin/gquery /usr/local/bin/gquery
CMD ["gquery"]