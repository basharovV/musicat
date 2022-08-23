<script>
	import { onMount, setContext, createEventDispatcher } from 'svelte';
	import { fade } from 'svelte/transition';

	export let x;
	export let y;
	
	// whenever x and y is changed, restrict box to be within bounds
	$: (() => {
		if (!menuEl) return;
		
		const rect = menuEl.getBoundingClientRect();
		x = Math.min(window.innerWidth - rect.width, x);
		if (y > window.innerHeight - rect.height) y -= rect.height;
	})();
	
	const dispatch = createEventDispatcher();	
	
	setContext('menu', {
		dispatchClick: () => dispatch('click')
	});
	
	let menuEl;
	function onPageClick(e) {
		if (e.target === menuEl || menuEl.contains(e.target)) return;
		dispatch('clickoutside');
	}
</script>

<style>
	div {
		position: absolute;
		display: grid;
		border: 1px solid rgba(255, 255, 255, 0.23);
		box-shadow: 2px 2px 5px 0px #0002;
        border-radius: 8px;
		background-color: #20202296;
        backdrop-filter: blur(8px);
        padding: 3px;
        font-family: -apple-system, 'Helvetica Neue', sans-serif;
        font-weight: 400;
        box-shadow: 10px 10px 10px 0px #0002;
	}
</style>

<svelte:body on:click={onPageClick} />

<div transition:fade={{ duration: 100 }} bind:this={menuEl} style="top: {y}px; left: {x}px;">
	<slot />
</div>