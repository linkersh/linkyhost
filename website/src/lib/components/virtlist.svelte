<script lang="ts">
	import { globalScrollTop, type ImageRow } from '$lib';
	import { onDestroy, onMount } from 'svelte';

	let { rows }: { rows: ImageRow[] } = $props();

	let viewportHeight = $state(0);
	let visibleItems: ImageRow[] = $state([]);
	let startIndex = $state(0);
	let scrollTop = $state(0);
	let endIndex = $state(0);
	let container: HTMLDivElement;

	$effect(() => {
		rows;
		recalculateVisibleRange();
		updateVisibleItems();
	});

	let unsubScroll: () => void | undefined;

	onMount(() => {
		updateViewportHeight();
		recalculateVisibleRange();
		updateVisibleItems();

		window.addEventListener('resize', updateViewportHeight);

		unsubScroll = globalScrollTop.subscribe((gScrollTop) => {
			if (gScrollTop === undefined || !container) return;

			scrollTop = Math.max(0, gScrollTop - (container?.offsetTop ?? 0));
			recalculateVisibleRange();
			updateVisibleItems();
		});
	});

	onDestroy(() => {
		unsubScroll?.();
		window.removeEventListener('resize', updateViewportHeight);
	});

	function updateViewportHeight() {
		viewportHeight = window.innerHeight;
		recalculateVisibleRange();
		updateVisibleItems();
	}

	function recalculateVisibleRange() {
		const currentScrollTop = scrollTop;
		const currentViewportHeight = viewportHeight;

		startIndex = calculateUntilHeight(currentScrollTop);
		endIndex = calculateUntilHeight(currentScrollTop + currentViewportHeight);
	}

	function updateVisibleItems() {
		const buffer = 1;
		const adjustedStart = Math.max(0, startIndex - buffer);
		const adjustedEnd = endIndex + buffer;
		visibleItems = rows.slice(adjustedStart, adjustedEnd);
	}

	function calculateUntilHeight(heightTarget: number) {
		let index = 0;
		let accumulatedHeight = 0;
		for (const row of rows) {
			if (accumulatedHeight >= heightTarget) {
				break;
			}
			accumulatedHeight += row.height;
			index++;
		}

		index = 0;
		accumulatedHeight = 0;
		for (const row of rows) {
			if (accumulatedHeight >= heightTarget) {
				return index;
			}
			accumulatedHeight += row.height;
			index++;
		}
		return rows.length;
	}

	function calculateTotalHeight() {
		let h = 0;
		for (const row of rows) {
			h += row.height;
		}
		return h;
	}

	function calculateHeightUntil(index: number) {
		let h = 0;
		for (let i = 0; i < index && i < rows.length; i++) {
			h += rows[i].height;
		}
		return h;
	}
</script>

<div bind:this={container} class="relative w-full" style="height: {calculateTotalHeight()}px">
	{#each visibleItems as item (item.index)}
		<div
			class="absolute w-full"
			style="top: {calculateHeightUntil(item.index)}px; height: {item.height}px;"
		>
			<div style="height: {item.height}px">
				<div
					class="border-background flex w-full flex-row gap-1 border-4"
					style="font-size: 0; line-height: 0; height: {item.height}px;"
				>
					{#each item.files as image (image.id)}
						<div
							style="
										width: {image.width}px;
										height: {image.height}px;
										display: inline-block;
										overflow: hidden;"
							class="border border-zinc-200 bg-zinc-700 shadow-sm"
						>
							<img
								loading="lazy"
								src={`https://picsum.photos/${image.height}/${image.width}`}
								style="width: 100%; height: 100%; object-fit: cover;"
								alt={'Gallery image'}
								width={image.width}
								height={image.height}
							/>
						</div>
					{/each}
				</div>
			</div>
		</div>
	{/each}
</div>
