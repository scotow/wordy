export function longpress(node, duration = 500) {
    let timer;

    function start(_event) {
        timer = setTimeout(() => {
            node.dispatchEvent(new CustomEvent('longpress'));
        }, duration);
    }

    function cancel() {
        clearTimeout(timer);
    }

    node.addEventListener('pointerdown', start);
    node.addEventListener('pointerup', cancel);
    node.addEventListener('pointermove', cancel);
    node.addEventListener('pointerleave', cancel);

    return {
        update(newDuration) {
            duration = newDuration;
        },
        destroy() {
            node.removeEventListener('pointerdown', start);
            node.removeEventListener('pointerup', cancel);
            node.removeEventListener('pointermove', cancel);
            node.removeEventListener('pointerleave', cancel);
            clearTimeout(timer);
        }
    };
}