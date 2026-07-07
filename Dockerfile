FROM node:26-slim AS frontend-builder

COPY frontend /app/frontend

WORKDIR /app/frontend
RUN npm install && npm run build

#------------

FROM rust:1.96-slim AS builder

COPY . /app
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist

WORKDIR /app
RUN cargo build --release

#------------

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/wordy /wordy

ENTRYPOINT ["/wordy"]