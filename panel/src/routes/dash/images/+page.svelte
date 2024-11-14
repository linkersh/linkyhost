<script lang="ts">
	import Input from '$lib/components/ui/input/input.svelte';
	import { onMount } from 'svelte';

	let imageQuery = $state('');
	let showingSearchResults = $state(false);
	let shownImages = $state([]);

	$effect(() => {
		handleQuery(imageQuery);
	});

	async function handleQuery(query: string) {
        query = query.trim();
        if (query.length === 0) return;

        console.log(query);

    }

	// onMount(() => {
	// 	for (let i = 1; i < 14; i++) {
	// 		images.push(`image${i}.png`);
	// 	}

	// 	console.log(images);
	// });
</script>

<div class="flex flex-row items-center">
	<Input bind:value={imageQuery} placeholder="Search images..." />
</div>

<div class="imageGallery my-6">
	<!-- {#each images as image}
		<img src="/{image}" alt="" />
	{/each} -->
</div>

<style>
	.imageGallery {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		grid-template-rows: masonry;
	}
</style>
