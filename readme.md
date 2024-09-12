# Redis-backed Page Service

This is a simple Axum-based web service that stores and retrieves HTML content from Redis. The service allows for creating and fetching pages using Redis as a data store. Each page has a 30-day expiration.

## Features

- **GET `/p/:path`**: Retrieves an HTML page stored in Redis at the given path.
- **POST `/create_page/:path`**: Creates or updates an HTML page at the given path. Requires an `Authorization` header to be set.

## Technologies

- **[Axum](https://docs.rs/axum/latest/axum/)**: A web framework for async Rust.
- **[Redis](https://docs.rs/redis/latest/redis/)**: Redis client for storing and retrieving page data.
- **[Tokio](https://docs.rs/tokio/latest/tokio/)**: Asynchronous runtime for handling concurrent connections.
- **[Tracing](https://docs.rs/tracing/latest/tracing/)**: For structured, leveled logging.

## Endpoints

### GET `/p/:path`

Retrieves the HTML content stored in Redis for the given `path`.

- **Path Parameters**:
  - `:path` - The path identifying the stored page.

- **Response**:
  - **200 OK**: HTML content is returned.
  - **404 Not Found**: No page found for the specified path.
  - **500 Internal Server Error**: Error accessing Redis or other server errors.

### POST `/create_page/:path`

Creates or updates an HTML page at the given `path`. Requires a valid `Authorization` header.

- **Headers**:
  - `Authorization`: Must match the `AUTH_TOKEN` environment variable.
  
- **Body**: Raw HTML content for the page.
  
- **Response**:
  - **200 OK**: Page is successfully created/updated.
  - **401 Unauthorized**: Missing or invalid `Authorization` token.
  - **500 Internal Server Error**: Error accessing Redis or other server errors.

## Environment Variables

- `REDIS_URL`: URL for connecting to the Redis instance. Defaults to `redis://127.0.0.1/`.
- `AUTH_TOKEN`: Token used to authenticate requests to the `POST /create_page/:path` endpoint.
- `HOST`: The host the service will bind to. Defaults to `127.0.0.1`.
- `PORT`: The port the service will bind to. Defaults to `3000`.
    