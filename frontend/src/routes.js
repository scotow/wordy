import Home from './Home.svelte';
import Game from './Game.svelte';

export default {
    '/': Home,
    '/games/:id': Game,
}