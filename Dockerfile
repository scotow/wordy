FROM node:26-slim AS frontend-builder

WORKDIR /app/frontend
COPY ./frontend .

RUN npm install && npm run build

#------------

FROM rust:1.96-slim AS builder

WORKDIR /app
COPY . .
COPY --from=frontend-builder /app/frontend/dist
RUN cargo build --release

#------------

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/dropit /dropit

ENTRYPOINT ["/dropit"]