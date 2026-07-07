use axum::{
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

/// The compiled Svelte app. `frontend/dist` is baked into the binary at build
/// time, so the shipped executable needs no files on disk and no Node runtime.
#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Frontend;

/// Serve embedded frontend files, falling back to `index.html` for unknown
/// paths so client-side routing (a single-page app) works on deep links.
pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Frontend::get(path) {
        Some(file) => serve(path, file),
        // Unknown path: hand back the SPA shell. If even that is missing the
        // frontend simply hasn't been built yet.
        None => match Frontend::get("index.html") {
            Some(file) => serve("index.html", file),
            None => (
                StatusCode::NOT_FOUND,
                "frontend not built — run `npm run build` in ./frontend (or just `cargo build`)",
            )
                .into_response(),
        },
    }
}

fn serve(path: &str, file: rust_embed::EmbeddedFile) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    (
        [(header::CONTENT_TYPE, mime.as_ref())],
        file.data.into_owned(),
    )
        .into_response()
}
