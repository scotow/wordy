<script>
    import {onMount} from "svelte";
    import Card from './Card.svelte';
    import {Timer} from './timer.svelte.js';
    import {longpress} from './longpress.js';
    import {baseUrl} from "./api.js";
    import {replace} from 'svelte-spa-router';

    const {
        params = {}
    } = $props();

    // Game phases:
    //   'idle'    – a single start card; swipe it to begin (direction picks mode)
    //   'playing' – the deck + a running timer
    //   'over'    – the deck has swept away and a recap card shows the final score
    let phase = $state('idle');
    // Set when a game starts: left swipe shows the exclusion hints, right hides them.
    let showExclusions = $state(true);

    let id = $derived(params.id);
    let cards = $state([]);

    let startX = $state(0);
    let startY = $state(0);

    let score = $state(0);
    let skips = $state(3);
    let idx = 0;

    // True only while a card is being dragged. The dragged card snaps instantly
    // to the finger; every other motion is animated by each card's Spring.
    let dragging = $state(false);

    // Drag offsets for the single start/recap card (only one is shown at a time).
    let menu = $state({dx: 0, dy: 0, wouldWin: 0, wouldPass: 0, commiting: false});

    const DURATION = 90_000;
    // autostart:false → the clock stays paused until a game begins; onExpire
    // triggers the end-of-round sweep.
    const timer = new Timer(DURATION, {autostart: false, onExpire: endGame});

    let topCard = $derived.by(() => {
        for (let i = 0; i < cards.length; i++) {
            if (!cards[i].commiting) {
                return cards[i];
            }
        }
    });

    // A flick this fast (px per millisecond) commits the card even if it hasn't
    // traveled the full distance threshold — so a quick swipe doesn't have to be
    // dragged almost all the way across the screen.
    const FLICK_VELOCITY = 0.5;

    // The card commits past 200px of drag, or within 50px of either screen edge.
    // `commitProgress` turns "how close to committing" into a 0→1 opacity for the
    // badge, but only over the final RAMP pixels, so the badge stays hidden
    // through most of the drag and fades in just before release would validate.
    const RAMP = 60;

    function commitProgress(dx, clientX) {
        const distance = (Math.abs(dx) - (200 - RAMP)) / RAMP;
        // Edge proximity only counts when actually dragging toward that edge.
        const leftEdge = dx < 0 ? (50 + RAMP - clientX) / RAMP : 0;
        const rightEdge = dx > 0 ? (clientX - (document.body.clientWidth - 50 - RAMP)) / RAMP : 0;
        const magnitude = Math.max(0, Math.min(1, Math.max(distance, leftEdge, rightEdge)));
        // Negative on the left (pass), positive on the right (win).
        return dx < 0 ? -magnitude : magnitude;
    }

    // Generic swipe: drives `card`'s offsets while dragging, then either flies it
    // off-screen — calling onCommit(-1) for a left swipe, onCommit(1) for right —
    // or snaps it back to rest. Shared by the deck cards and the start/recap card.
    function drag(event, card, onCommit) {
        dragging = true;
        startX = event.clientX;
        startY = event.clientY;

        // Track pointer velocity from the most recent move so pointerup can tell a
        // flick from a slow drag.
        let lastX = event.clientX;
        let lastY = event.clientY;
        let lastT = event.timeStamp;
        let vx = 0;
        let vy = 0;

        window.onpointermove = (event) => {
            card.dx = event.clientX - startX;
            card.dy = event.clientY - startY;
            const progress = commitProgress(card.dx, event.clientX);
            card.wouldWin = Math.max(0, progress);
            card.wouldPass = Math.max(0, -progress);

            const dt = event.timeStamp - lastT;
            if (dt > 0) {
                vx = (event.clientX - lastX) / dt;
                vy = (event.clientY - lastY) / dt;
                lastX = event.clientX;
                lastY = event.clientY;
                lastT = event.timeStamp;
            }
        }

        window.onpointerup = (event) => {
            window.onpointermove = undefined;
            window.onpointerup = undefined;
            dragging = false;

            // If the finger paused before lifting, the last sampled velocity is
            // stale — treat it as a release, not a flick.
            if (event.timeStamp - lastT > 100) {
                vx = 0;
                vy = 0;
            }

            const distanceCommit = Math.abs(card.dx) > 200 || event.clientX < 50 || event.clientX > document.body.clientWidth - 50;
            const flickCommit = Math.abs(vx) > FLICK_VELOCITY;

            if (distanceCommit || flickCommit) {
                // Fly off-screen in the committed direction. Use the flick
                // direction when flicking, otherwise the drag offset — never the
                // raw release position.
                const dir = (flickCommit ? vx : card.dx) < 0 ? -1 : 1;
                card.dx = dir * Math.max(window.innerWidth, 500);
                card.dy += ((flickCommit ? vy : card.dy) < 0 ? -10 : 10);
                card.commiting = true;
                onCommit(dir, card.difficulty);
            } else {
                card.wouldWin = 0;
                card.wouldPass = 0;
                // Snap back to rest (animated by the card's Spring).
                card.dx = 0;
                card.dy = 0;
            }
        }
    }

    // Swipe on the top deck card: right scores, left spends a pass (or costs a
    // point once passes run out). Then drop the card and replenish the deck.
    // Resolve a committed card: right (1) scores, left (-1) spends a pass or, once
    // none are left, costs a point. Then drop the card and replenish the deck.
    function resolveCard(dir, difficulty) {
        switch (dir) {
            case 1:
                score += difficulty;
                break;
            case -1:
                if (skips > 0) {
                    skips -= 1;
                } else {
                    score = Math.max(0, score - 1);
                }
                break;
        }

        setTimeout(() => {
            cards.shift();
        }, 500);

        addCard(1);
    }

    function onDeckPointerDown(event) {
        // Grab a stable reference: `topCard` changes the moment we commit below.
        const card = topCard;
        if (!card) return;
        drag(event, card, resolveCard);
    }

    // Tap the timer to pause; tap again to resume. Pausing locks the top card
    // (it can't be swiped while the clock is stopped), and resuming discards that
    // card — so pausing to study the word forfeits it rather than buying time.
    function togglePause() {
        if (phase !== 'playing') return;
        if (timer.paused) {
            timer.resume();
            discardTop(-1);
            setTimeout(() => {
                discardTop(0);
            }, 300);
        } else {
            timer.pause();
        }
    }

    // Discard the top card exactly as a left swipe would: flash the pass/malus
    // stamp, fly it off to the left, and burn a pass (or lose a point).
    function discardTop(penality) {
        const card = topCard;
        if (!card) return;
        card.wouldPass = penality ? 1 : 0;
        card.commiting = true;
        card.dx = -Math.max(window.innerWidth, 500);
        resolveCard(penality ? -1 : 0, card.difficulty);
    }

    // Swipe on the start/recap card: begin a game (direction picks the mode) or,
    // from the recap, return to the start card.
    function onMenuPointerDown(event) {
        drag(event, menu, (dir) => {
            if (phase === 'idle') {
                startGame(dir === -1); // left → exclusions visible, right → hidden
            } else if (phase === 'over') {
                restart();
            }
        });
    }

    function buildCard(cardData, deal) {
        return {
            idx: idx++,
            number: cardData.number,
            colors: cardData.colors,
            category: cardData.category,
            word: cardData.word,
            difficulty: cardData.difficulty,
            exclusions: cardData.exclusions,
            dx: 0,
            dy: 0,
            wouldWin: 0,
            wouldPass: 0,
            commiting: false,
            deal, // larger spring entrance for the opening hand
        };
    }

    // Replenish a single card after a swipe (subtle entrance).
    async function addCard(count) {
        const resp = await fetch(`${baseUrl()}/games/${id}/words?count=${count}`);
        const cardsData = await resp.json();
        for (const cardData of cardsData) {
            cards.push(buildCard(cardData, false));
        }
    }

    const INITIAL_CARDS = 5;
    // Gap between each dealt card mounting, for the staggered "deal" effect.
    const DEAL_STAGGER = 90;

    // Deal the opening hand: fetch all cards up front, then mount them one at a
    // time so each springs up into the stack in turn.
    async function dealInitial() {
        // if (!id) await newGame();
        const resp = await fetch(`${baseUrl()}/games/${id}/words?count=${INITIAL_CARDS}`);
        const cardsData = await resp.json();
        cardsData.forEach((cardData, i) => {
            setTimeout(() => cards.push(buildCard(cardData, true)), i * DEAL_STAGGER);
        });
    }

    function resetMenu() {
        menu.dx = 0;
        menu.dy = 0;
        menu.wouldWin = 0;
        menu.wouldPass = 0;
        menu.commiting = false;
    }

    function startGame(withExclusions) {
        showExclusions = withExclusions;
        // Let the start card finish flying off, then drop into the game and deal.
        setTimeout(() => {
            phase = 'playing';
            score = 0;
            skips = 3;
            resetMenu();
            timer.reset(DURATION);
            dealInitial();
        }, 350);
    }

    function endGame() {
        phase = 'over';
        // Sweep the deck off the bottom of the screen (staggered), then clear it
        // and let the recap card spring in.
        const count = cards.length;
        cards.forEach((card, i) => {
            setTimeout(() => {
                card.commiting = true;
                card.dy = window.innerHeight;
            }, i * 60);
        });
        setTimeout(() => {
            cards = [];
            resetMenu();
        }, count * 60 + 500);
    }

    function restart() {
        // Recap card flies off, then we return to the start card.
        setTimeout(() => {
            phase = 'idle';
            resetMenu();
        }, 350);
    }

    async function newGame() {
        const resp = await fetch(`${baseUrl()}/games/${id}`);
        if (resp.status !== 200) {
            await replace('/');
        }
    }

    onMount(newGame);
</script>

<div
        class="header"
        class:hidden={phase !== 'playing'}
        use:longpress={600}
        onlongpress={endGame}
>
    <div class="section score">
        <div class="title">Score</div>
        <div class="content">
            <div class="text">{score}</div>
        </div>
    </div>
    <div class="section skips">
        <div class="title">Passes</div>
        <div class="content">
            {#each {length: 3} as _, i}
                <div class="skip" class:available={skips > i}></div>
            {/each}
        </div>
    </div>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div onclick={togglePause} class="section timer" class:paused={timer.paused}>
        <div class="title">Temps</div>
        <div class="content">
            <div class="text">{Math.floor(timer.seconds / 60)}:{String(timer.seconds % 60).padStart(2, '0')}</div>
            <div class="pause"></div>
        </div>
    </div>
</div>
<div class="deck">
    {#if phase === 'idle'}
        <Card
                variant="start"
                idx={0}
                dx={menu.dx}
                dy={menu.dy}
                dz={0}
                enter={80}
                onpointerdown={onMenuPointerDown}
                instant={dragging}
        />
    {/if}

    {#each cards as card, i (card.idx)}
        <Card
                variant="word"
                idx={card.idx}
                dx={card.dx}
                dy={card.dy}
                dz={cards.length - i}
                enter={card.deal ? 320 : 14}
                category={card.category}
                number={card.number}
                word={card.word}
                difficulty={card.difficulty}
                exclusions={card.exclusions}
                colors={card.colors}
                {showExclusions}
                onpointerdown={card.idx === topCard?.idx && timer.running ? onDeckPointerDown : undefined}
                instant={dragging && card.idx === topCard?.idx}
                commiting={card.commiting}
                wouldWin={card.wouldWin}
                wouldPass={card.wouldPass}
                {skips}
        />
    {/each}

    {#if phase === 'over' && cards.length === 0}
        <Card
                variant="recap"
                {score}
                idx={0}
                dx={menu.dx}
                dy={menu.dy}
                dz={0}
                enter={80}
                onpointerdown={onMenuPointerDown}
                instant={dragging}
        />
    {/if}
</div>

<style>
    .header {
        height: 50px;
        margin: 20px;
        display: flex;
        justify-content: space-between;
        border: 1px solid #DDD0AC;
        border-radius: 14px;
        background-color: #FBF8F1;
        transition: transform 0.5s cubic-bezier(0.22, 1, 0.36, 1), opacity 0.4s ease;
    }

    /* Tucked up out of view in the idle/over phases; slides down when playing. */
    .header.hidden {
        transform: translateY(-110px);
        opacity: 0;
        pointer-events: none;
    }

    .header > .section {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        flex-direction: column;
    }

    .header > .section:nth-child(2) {
        border-left: 1px solid #DDD0AC;
        border-right: 1px solid #DDD0AC;
    }

    .header > .section > .title {
        line-height: 1;
        font-size: 10px;
        font-family: var(--sans-serif);
        letter-spacing: 1px;
        color: #8C7B54;
        text-transform: uppercase;
    }

    .header > .section > .content > .text {
        line-height: 1;
        font-size: 18px;
        font-family: var(--serif);
        letter-spacing: 1px;
        color: #4A3F2C;
        text-transform: uppercase;
    }

    .header > .section.skips > .content {
        display: flex;
        align-items: center;
        gap: 4px;
        height: 18px;
    }

    .header > .section.skips > .content > .skip {
        width: 12px;
        height: 12px;
        border: 1px solid #8C7B54;
        border-radius: 4px;
    }

    .header > .section.skips > .content > .skip.available {
        background-color: #fcc48b;
    }

    .header > .section.timer > .content > .text {
        font-variant-numeric: tabular-nums;
    }

    .deck {
        flex: 1;
        display: flex;
        justify-content: center;
        align-items: center;
    }
</style>
