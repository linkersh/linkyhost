<script lang="ts">
	import Sidebar from '$lib/components/sidebar.svelte';
	import { blur } from 'svelte/transition';
	import { quadIn } from 'svelte/easing';
	import { page } from '$app/state';

	let { children } = $props();

	// Track the current page path to trigger animations on change
	let currentPath = $derived(page.url.pathname);
</script>

<!-- Sidebar (no animation) -->
<div class="fixed left-0 top-0 h-screen w-[14rem]">
	<Sidebar></Sidebar>
</div>

<!-- Body with blur animation only on page changes -->
{#key currentPath}
	<div
		in:blur={{ duration: 200, easing: quadIn }}
		class="fixed left-56 top-0 h-screen w-[calc(100vw-14rem)] p-4"
	>
		{@render children()}
	</div>
{/key}
