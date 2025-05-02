<script lang="ts">
	import { onMount } from 'svelte';
	import ImageGroup from './ImageGroup.svelte';
	import { getBuckets, type TimeBucket } from '$lib/api/files';
	import { globalScrollTop } from '$lib';

	let buckets: TimeBucket[] = $state([]);
	let scrollContainer: HTMLDivElement;

	onMount(async () => {
		buckets = await getBuckets('image');

		// setTimeout(() => {
		// 	scrollIntoBucket.set(buckets[5].date.toISOString());
		// }, 5000);
	});

	function onscroll(e: UIEvent & { currentTarget: EventTarget & HTMLDivElement }) {
		globalScrollTop.set(e.currentTarget.scrollTop);
	}
</script>

<div {onscroll} bind:this={scrollContainer} class="h-full w-full overflow-y-auto">
	{#each buckets as group, i}
		<ImageGroup date={group.date} bucketKey={group.date.toISOString()}></ImageGroup>
	{/each}
</div>
