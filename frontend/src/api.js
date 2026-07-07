export function baseUrl() {
    if (window.location.host === '10.0.1.1:5173') {
        return "http://10.0.1.1:8080";
    } else {
        return `http://${window.location.host}`;
    }
}