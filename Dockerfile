# syntax=docker/dockerfile:1

FROM node:24-bookworm-slim AS frontend

WORKDIR /src/frontend

COPY frontend/package.json frontend/package-lock.json frontend/.npmrc ./
RUN npm ci

COPY frontend/index.html frontend/tsconfig.json frontend/vite.config.ts ./
COPY frontend/src ./src
RUN npm run build

FROM rust:1.99-bookworm AS backend

WORKDIR /src/backend

COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/.cargo ./.cargo
COPY backend/src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN groupadd --system --gid 10001 listquiz \
    && useradd --system --uid 10001 --gid listquiz --home-dir /app --no-create-home listquiz \
    && mkdir -p /app/frontend/dist /app/data \
    && chown -R listquiz:listquiz /app

COPY --from=frontend --chown=listquiz:listquiz /src/frontend/dist /app/frontend/dist
COPY --from=backend --chown=listquiz:listquiz /src/backend/target/release/listquiz /app/listquiz

WORKDIR /app

ENV LISTQUIZ_ROOT=/app \
    LISTQUIZ_DIST=/app/frontend/dist \
    LISTQUIZ_DB=/app/data/listquiz.db

EXPOSE 8080
VOLUME ["/app/data"]

USER listquiz
CMD ["./listquiz"]
