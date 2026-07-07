// A countdown timer as a reusable reactive class.
//
// Source of truth is an absolute `deadline` timestamp, not an accumulator, so
// the displayed value stays correct even if interval ticks arrive late or the
// tab gets throttled in the background. The interval only nudges `now` to
// trigger re-renders; the real arithmetic is `deadline - now`.
export class Timer {
    // Absolute moment the countdown hits zero.
    #deadline = $state(0);
    // Bumped every tick purely to drive reactivity.
    #now = $state(0);
    // While paused, holds the frozen remaining ms; null means running. Reading
    // it inside the $effect below is what makes pausing tear down the interval.
    #frozen = $state(null);
    // Set once the countdown reaches zero; also stops the interval.
    #expired = $state(false);
    // Fired once when the countdown reaches zero.
    #onExpire;

    constructor(duration, {autostart = true, onExpire} = {}) {
        this.#onExpire = onExpire;
        this.#now = Date.now();
        this.#deadline = this.#now + duration;
        // Start paused at full duration when autostart is off, so the clock only
        // begins on the first reset()/resume().
        if (!autostart) this.#frozen = duration;

        $effect(() => {
            // Only run an interval while actually counting. Pausing (#frozen set)
            // or expiring re-runs this effect, and the cleanup clears the timer —
            // reactive teardown, not just unmount cleanup.
            if (this.#frozen !== null || this.#expired) return;

            const id = setInterval(() => this.#tick(), 100);
            return () => clearInterval(id);
        });
    }

    #tick() {
        this.#now = Date.now();
        if (this.#now >= this.#deadline) {
            this.#now = this.#deadline; // clamp so `remaining` lands exactly on 0
            this.#expired = true;       // stops the interval via the $effect
            this.#onExpire?.();
        }
    }

    get running() {
        return this.#frozen === null && !this.#expired;
    }

    get paused() {
        return this.#frozen !== null;
    }

    get expired() {
        return this.#expired;
    }

    get remaining() {
        return this.#frozen ?? Math.max(0, this.#deadline - this.#now);
    }

    get seconds() {
        return Math.ceil(this.remaining / 1000);
    }

    pause() {
        if (!this.running) return;
        this.#frozen = this.remaining;
    }

    resume() {
        if (this.#frozen === null) return;
        // Re-anchor the deadline so the countdown picks up where it left off.
        this.#deadline = Date.now() + this.#frozen;
        this.#now = Date.now();
        this.#frozen = null;
    }

    toggle() {
        this.running ? this.pause() : this.resume();
    }

    // (Re)start a fresh countdown of `duration` ms.
    reset(duration) {
        this.#now = Date.now();
        this.#deadline = this.#now + duration;
        this.#frozen = null;
        this.#expired = false;
    }
}
