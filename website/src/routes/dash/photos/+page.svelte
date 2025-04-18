<script lang="ts">
	import { getBucketFiles, getBuckets, type APIFile, type TimeBucket } from '$lib/api/files';
	import { onMount, tick } from 'svelte';
	import PageTitle from '$lib/components/page-title.svelte';
	import Timeline from '$lib/components/timeline.svelte';
	import { getGallery, type BucketWithFiles } from '$lib/gallery';
	import MasonryLayout from './MasonryLayout.svelte';
	import { writable, type Writable } from 'svelte/store';

	let files: BucketWithFiles[] = $state([]);
	let scrollBucketStore: Writable<string | undefined> = writable();

	async function fetchFiles(bucket?: TimeBucket) {
		const gallery = await getGallery();
		files = await gallery.getLoadedBuckets();

		await tick();

		if (bucket) {
			scrollBucketStore.set(bucket.date);
		}
	}

	onMount(async () => {
		await fetchFiles();

		const gallery = await getGallery();
		gallery.focuson = async (currentBucket) => {
			await fetchFiles(currentBucket);
		};
	});
</script>

<PageTitle>Photos</PageTitle>

<div class="flex flex-row justify-between gap-2">
	<div class="h-[calc(100vh-4rem)] w-screen overflow-y-auto">
		<MasonryLayout buckets={files} {scrollBucketStore}></MasonryLayout>
	</div>

	<!-- {#if buckets.length > 0} -->
	<Timeline></Timeline>
	<!-- {/if} -->
</div>
