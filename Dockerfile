FROM rust:alpine AS builder

RUN apk add musl-dev make
WORKDIR /src/gquery
COPY . .
RUN cargo build --release

FROM scratch
COPY --from=builder /src/gquery/target/release/gquery /usr/bin/gquery
CMD ["gquery"]