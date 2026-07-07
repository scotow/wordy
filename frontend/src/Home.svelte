<script>
    import {onMount} from "svelte";
    import {baseUrl} from "./api.js";
    import {replace} from 'svelte-spa-router';

    onMount(async () => {
        const resp = await fetch(`${baseUrl()}/games`, {method: 'POST'});
        const {id} = await resp.json();
        await replace(`/games/${id}`);
    });
</script>