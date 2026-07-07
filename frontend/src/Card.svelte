<script>
    import {untrack} from 'svelte';
    import {cubicOut} from 'svelte/easing';
    import {Spring} from 'svelte/motion';

    // Discreet entrance for a freshly-dealt card: a soft fade only. The gentle
    // upward slide into the deck is handled by the Spring's starting offset
    // below (animating transform here would fight the spring). Kept subtle on
    // purpose so new cards appear without drawing the eye.
    function appear(_node, {duration = 450} = {}) {
        return {
            duration,
            easing: cubicOut,
            css: (t) => `opacity: ${t}`,
        };
    }

    let {
        idx,
        dx,
        dy,
        dz,
        // 'word' is a normal play card; 'start' and 'recap' are the single menu
        // cards shown between rounds. The chrome (border, spring, drag) is shared;
        // only the inner content and the stamps differ.
        variant = 'word',
        score = 0,
        // Whether the exclusion hints are shown (game-wide difficulty setting).
        showExclusions = true,
        // How far below its resting slot the card starts on mount.
        enter = 14,
        category = '',
        number = 0,
        word = '',
        difficulty = 0,
        exclusions = [],
        colors,
        onpointerdown,
        instant,
        commiting = false,
        wouldWin = 0,
        wouldPass = 0,
        skips = 0,
    } = $props();

    // Per-card resting offsets so the stack looks hand-dropped. `idx` is fixed
    // for a given (keyed) card, so reading it once at mount is intentional.
    const baseX = untrack(() => (idx % 2 === 0 ? -1 : 1) * 12);
    const baseRot = untrack(() => {
        switch (idx % 5) {
            case 0:
                return -2;
            case 1:
                return 1;
            case 2:
                return -1;
            case 3:
                return 2;
            default:
                return 0;
        }
    });

    // The visual offset is animated by a Spring (driven via requestAnimationFrame),
    // not by a CSS transition. There's no transition to switch on/off, so the
    // Safari "transition sticks during a drag" problem can't happen.
    // untrack: we deliberately want the resting position *at mount* as the
    // spring's starting value, not a reactive read. The card starts `enter` px
    // below its resting slot, then the $effect below springs it up into place —
    // a small offset for a replenished card (a discreet "slid into the deck"),
    // a large one for a freshly dealt hand (rises up from off-screen).
    const enterOffset = untrack(() => enter);

    const pos = new Spring(
        untrack(() => ({x: dx + baseX, y: dz * 14 + dy + enterOffset, rot: dx * 0.06 + baseRot})),
        {stiffness: 0.15, damping: 0.7}, // tweak these two for the feel
    );

    // Where the card *should* be, given the current drag offset and stack depth.
    let target = $derived({
        x: dx + baseX,
        y: dz * 14 + dy,
        rot: dx * 0.06 + baseRot,
    });

    $effect(() => {
        // While dragging the top card: snap instantly to the finger.
        // Otherwise (snap-back, fly-off, stack settling): animate.
        pos.set(target, {instant});
    });

    // The pass/malus stamp choice is latched the instant the card commits: a left
    // swipe spends a pass, so `skips` drops, but we keep the stamp the player saw
    // on release rather than flipping it mid-flight. Until commit, it tracks live
    // skips. (A latch on a condition can't be a pure $derived — hence the effect.)
    // svelte-ignore state_referenced_locally
    let stampSkips = $state(skips);
    $effect(() => {
        if (!commiting) stampSkips = skips;
    });
</script>

<div
        in:appear
        class="card {variant === 'word' ? 'word' : 'menu'}"
        style:transform="translate({pos.current.x}px, {pos.current.y}px) rotate({pos.current.rot}deg)"
        style:z-index={dz}
        style:background-color={colors?.background ?? '#FBF8F1'}
        style:border-color={colors?.border ?? '#DDD0AC'}
        role="group"
        {onpointerdown}
>
    {#if variant === 'word'}
        {#if wouldWin > 0}
            <div class="stamp positive" style:opacity={wouldWin * 0.88}>+{difficulty}</div>
        {/if}
        {#if wouldPass > 0}
            <div class="stamp negative" style:opacity={wouldPass * 0.88}>{stampSkips > 0 ? 'Pass' : '-1'}</div>
        {/if}
        <div class="header" style:color={colors.header}>
            <div class="category">{category}</div>
            <div class="number">{number}</div>
        </div>
        <div class="main">
            <div class="word" style:color={colors.word}>
                {word}
            </div>
            <div class="difficulty" style:color={colors.word}>
                {#each {length: difficulty} as _, i}
                    <div class="star">★</div>
                {/each}
            </div>
        </div>
        <div class="footer">
            {#if showExclusions}
                <div class="separator" style:background-color={colors.border}></div>
                <div class="exclusions" style:color={colors.exclusion}>
                    {#each exclusions as exclusion}
                        <div class="exclusion">
                            <div class="dot" style:background-color={colors.dot}></div>
                            <div class="word">{exclusion}</div>
                        </div>
                    {/each}
                </div>
            {/if}
        </div>
    {:else if variant === 'start'}
        <div class="title">Wordy</div>
        <div class="text">Glissez pour jouer</div>
        <div class="modes">
            <div class="mode">
                <div class="arrow">←</div>
                <div>Avec<br/>exclusions</div>
            </div>
            <div class="mode">
                <div class="arrow">→</div>
                <div>Sans<br/>exclusions</div>
            </div>
        </div>
    {:else if variant === 'recap'}
        <div class="text">Terminé</div>
        <div class="score">{score}</div>
        <div class="text">Glissez pour rejouer</div>
    {/if}
</div>

<style>
    .card {
        position: absolute;
        /* Claim all touch gestures for the drag: without this the browser
           treats a drag to the edge as a pan/zoom (and iOS ignores
           user-scalable=no, so the viewport meta can't prevent it). */
        touch-action: none;
        display: flex;
        flex-direction: column;
        width: calc(100vw - 100px);
        /*height: calc(100vh - 200px);*/
        max-width: 400px;
        max-height: 560px;
        aspect-ratio: 5 / 7;
        border-width: 1px;
        border-style: solid;
        border-radius: 24px;
        box-shadow: 0 18px 40px rgba(0, 0, 0, 0.45);

        &.word {
            padding: 14px;

            > .stamp {
                position: absolute;
                top: 70px;
                padding: 2px 10px;
                text-align: center;
                font-size: 40px;
                font-family: AlfaSlabOne;
                color: white;
                text-transform: uppercase;
                background-repeat: no-repeat;
                background-position: center;
                clip-path: polygon(
                        2% 4%, 6% 0%, 14% 3%, 22% 0%, 30% 2%, 40% 0%,
                        50% 3%, 60% 0%, 70% 2%, 80% 0%, 90% 3%, 98% 5%,
                        100% 14%, 97% 26%, 100% 40%, 97% 55%, 100% 70%,
                        97% 84%, 100% 96%, 92% 100%, 80% 97%, 68% 100%,
                        54% 97%, 40% 100%, 26% 97%, 14% 100%, 4% 96%,
                        0% 84%, 3% 70%, 0% 55%, 3% 40%, 0% 26%, 2% 14%
                );

                &.positive {
                    left: 20px;
                    background-color: #27ae60dd;
                    transform: rotate(-12deg);
                }

                &.negative {
                    right: 20px;
                    background-color: rgb(191 88 88 / 0.87);
                    transform: rotate(12deg);
                }
            }

            > .header {
                display: flex;
                justify-content: space-between;
                font-family: var(--sans-serif);
                font-size: 12px;
                font-weight: 500;

                > .category {
                    letter-spacing: 0.1em;
                    text-transform: uppercase;
                    text-wrap: pretty;
                }

                > .number {
                    margin-left: 10px;

                    &::before {
                        content: '#';
                    }
                }
            }

            > .main {
                flex: 1;
                display: flex;
                align-items: center;
                justify-content: center;
                flex-direction: column;
                gap: 10px;

                > .difficulty {
                    display: flex;

                    > .star {
                        font-size: 24px;
                    }
                }

                > .word {
                    text-align: center;
                    font-size: 38px;
                    font-weight: 400;
                    font-family: var(--serif);

                    &::first-letter {
                        text-transform: uppercase;
                    }
                }
            }

            > .footer {
                > .separator {
                    height: 1px;
                    margin-bottom: 14px;
                }

                > .exclusions {
                    display: flex;
                    flex-direction: column;
                    gap: 2px;

                    > .exclusion {
                        display: flex;
                        align-items: center;
                        gap: 6px;

                        > .dot {
                            width: 4px;
                            height: 4px;
                            border-radius: 50%;
                        }

                        > .word {
                            font-family: var(--sans-serif);
                            font-size: 14px;

                            &::first-letter {
                                text-transform: uppercase;
                            }
                        }
                    }
                }
            }
        }

        &.menu {
            gap: 28px;
            padding: 14px 30px;

            /*> .menu {*/
            /*    flex: 1;*/
            /*    display: flex;*/
            /*    flex-direction: column;*/
            align-items: center;
            justify-content: center;
            /*    text-align: center;*/


            > .title {
                font-family: var(--serif);
                font-size: 52px;
                letter-spacing: 2px;
                color: #4A3F2C;
            }

            > .score {
                font-family: var(--serif);
                font-size: 96px;
                line-height: 1;
                color: #4A3F2C;
            }

            > .text {
                font-family: var(--sans-serif);
                font-size: 13px;
                letter-spacing: 1px;
                text-transform: uppercase;
                color: #8C7B54;
            }

            > .modes {
                width: 100%;
                margin-top: 8px;
                display: flex;
                justify-content: space-between;
                font-family: var(--sans-serif);
                font-size: 12px;
                letter-spacing: 0.05em;
                text-transform: uppercase;
                color: #8C7B54;

                > .mode {
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    gap: 6px;
                    text-align: center;

                    > .arrow {
                        font-size: 22px;
                    }
                }
            }
        }
    }
</style>
