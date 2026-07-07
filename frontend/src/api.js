export function baseUrl() {
    // In dev (npm run dev), the Vite dev server and the backend run on
    // different ports of the same host. In prod the built single-file
    // frontend is served by the backend itself, so same-origin works.
    if (import.meta.env.DEV) {
        return `${window.location.protocol}//${window.location.hostname}:8080`;
    }
    return `${window.location.protocol}//${window.location.host}`;
}
