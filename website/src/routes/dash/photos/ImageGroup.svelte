<script lang="ts">
	import { PUBLIC_SERVER_URL } from '$env/static/public';
	import { scrollIntoBucket, type ImageFile, type ImageRow } from '$lib';
	import { getBucketFiles, type APIFile } from '$lib/api/files';
	import Virtlist from '$lib/components/virtlist.svelte';
	import { onDestroy, onMount, tick } from 'svelte';

	interface Props {
		date: Date;
		bucketKey: string;
	}

	const MIN_HEIGHT_PX = 320;
	const BATCH_SIZE = 12;

	let { bucketKey, date }: Props = $props();

	let isVisible = $state(false);
	let allFiles: APIFile[] = $state([]);
	let renderedFiles: APIFile[] = $state([]);
	let element: HTMLDivElement;
	let observer: IntersectionObserver | null = null;
	let isRendering = $state(false);
	let rafId: number | null = null;
	let measuredHeight: number | null = $state(null);
	let hasFetched = $state(false);

	async function renderNextBatch() {
		if (!isVisible || !isRendering) {
			console.log(
				`Stopping render for ${date.toISOString()}. Visible: ${isVisible}, Rendering: ${isRendering}`
			);
			isRendering = false;
			if (rafId) cancelAnimationFrame(rafId);
			rafId = null;
			return;
		}

		const currentLength = renderedFiles.length;
		const nextBatch = allFiles.slice(currentLength, currentLength + BATCH_SIZE);

		if (nextBatch.length > 0) {
			renderedFiles = [...renderedFiles, ...nextBatch];
			rafId = requestAnimationFrame(renderNextBatch);
		} else {
			console.log(`Finished rendering for ${date.toISOString()}`);
			isRendering = false;
			rafId = null;
			await tick();
			if (element && measuredHeight === null) {
				measuredHeight = element.clientHeight;
				console.log(`Measured height for ${date.toISOString()}: ${measuredHeight}px`);
			}
		}
	}
	$effect(() => {
		if (isVisible) {
			if (!hasFetched) {
				fetchFiles();
			}
			else if (allFiles.length > 0 && !isRendering && renderedFiles.length === 0) {
				startIncrementalRender();
			}
		} else {
			if (isRendering) {
				console.log(`Cancelling render due to visibility change: ${date.toISOString()}`);
				isRendering = false;
				if (rafId) {
					cancelAnimationFrame(rafId);
					rafId = null;
				}
			}
			
			if (renderedFiles.length > 0) {
				renderedFiles = [];
			}
		}
	});

	function startIncrementalRender() {
		if (isRendering || !allFiles.length || !isVisible || renderedFiles.length > 0) {
			isRendering = false; 
			return;
		}

		console.log(`Starting incremental render for ${date.toISOString()}`);
		isRendering = true;
		renderedFiles = [];
		renderNextBatch();
	}

	async function fetchFiles() {
		if (hasFetched) return;
		console.log(`Requesting files for: ${date.toISOString()}`);
		try {
			hasFetched = true;
			const loadedFiles = await getBucketFiles('image', date.toISOString());
			allFiles = loadedFiles;
			console.log(`Loaded ${allFiles.length} files for: ${date.toISOString()}`);

			if (isVisible && allFiles.length > 0 && !isRendering) {
				startIncrementalRender();
			}
		} catch (error) {
			console.error(`Failed to load files for ${date.toISOString()}:`, error);
			hasFetched = false;
		}
	}

	let unsub: () => void | undefined;
	onMount(() => {
		if (!element) return;

		const options = {
			root: element.parentElement,
			rootMargin: '500px 0px 500px 0px',
			threshold: 0
		};

		const intersectionCallback = (entries: IntersectionObserverEntry[]) => {
			entries.forEach((entry) => {
				if (entry.target === element) {
					const newVisibility = entry.isIntersecting;
					if (isVisible !== newVisibility) {
						console.log(`Bucket ${date.toISOString()} intersecting: ${newVisibility}`);
						isVisible = newVisibility;
					}
				}
			});
		};

		observer = new IntersectionObserver(intersectionCallback, options);
		observer.observe(element);

		unsub = scrollIntoBucket.subscribe((x) => {
			if (x === bucketKey) {
				element.scrollIntoView({ behavior: 'smooth', block: 'start' });
			}
		});
	});

	onDestroy(() => {
		if (observer && element) {
			console.log(`Unobserving ${date.toISOString()}`);
			observer.unobserve(element);
		}
		observer?.disconnect();
		unsub?.();
		if (rafId) {
			cancelAnimationFrame(rafId);
		}
	});

	function getPlaceholderHeight(): number {
		return measuredHeight ?? MIN_HEIGHT_PX;
	}

	let rows: ImageRow[] = [];

	const targetRowHeight = 320;
	const maxRowHeight = 400;
	const imageGap = 0;
	const rowGap = 0;
	const minImagesPerRow = 3;
	const maxImagesPerRow = 5;
	const maxSingleImageAspectRatio = 1.2;
	const fillThreshold = 0.5;

	function buildMasonry() {
		const containerWidth = element.clientWidth;
		console.log('container width:', containerWidth);

		let currentRowImages: APIFile[] = [];
		let currentRowWidthSum = 0;

		const getEffectiveWidth = (numImages: number) => {
			const gaps = Math.max(0, numImages - 1) * imageGap;
			return containerWidth - gaps;
		};

		console.log('total files:', allFiles.length);

		const shouldCreateRow = (numImages: number, widthSum: number, isLastImage: boolean) => {
			if (isLastImage && numImages > 0) return true;
			if (numImages >= maxImagesPerRow) return true;

			const effectiveWidth = getEffectiveWidth(numImages);
			const fillRatio = widthSum / effectiveWidth;

			return numImages >= minImagesPerRow && fillRatio >= fillThreshold;
		};

		for (let i = 0; i < allFiles.length; i++) {
			const img = allFiles[i];
			const aspectRatio = img.width && img.height ? img.width / img.height : 16 / 9;
			const scaledWidth = targetRowHeight * aspectRatio;

			currentRowImages.push(img);
			currentRowWidthSum += scaledWidth;

			const isLastImage = i === allFiles.length - 1;

			if (shouldCreateRow(currentRowImages.length, currentRowWidthSum, isLastImage)) {
				const effectiveContainerWidth = getEffectiveWidth(currentRowImages.length);
				let scale = effectiveContainerWidth / currentRowWidthSum;

				if (currentRowImages.length === 1) {
					const singleImage = currentRowImages[0];
					const singleAspectRatio =
						singleImage.width && singleImage.height
							? singleImage.width / singleImage.height
							: 16 / 9;

					const maxWidth = containerWidth;
					const desiredWidth = targetRowHeight * singleAspectRatio;
					let finalWidth = Math.min(maxWidth, desiredWidth);

					scale = finalWidth / desiredWidth;

					if (singleAspectRatio > maxSingleImageAspectRatio) {
						const cappedWidth = targetRowHeight * maxSingleImageAspectRatio;
						finalWidth = Math.min(finalWidth, cappedWidth);
						scale = finalWidth / desiredWidth;
					}
				}

				const resultingHeight = targetRowHeight * scale;
				if (resultingHeight > maxRowHeight) {
					scale *= maxRowHeight / resultingHeight;
				}

				const processedImages: ImageFile[] = currentRowImages.map((img) => {
					const imgAspectRatio = img.width && img.height ? img.width / img.height : 16 / 9;
					return {
						width: Math.floor(targetRowHeight * imgAspectRatio * scale),
						height: Math.floor(targetRowHeight * scale),
						id: img.id
					};
				});

				rows.push({
					files: processedImages,
					height: processedImages[0]!.height,
					index: rows.length
				});

				currentRowImages = [];
				currentRowWidthSum = 0;
			}
		}

		if (currentRowImages.length > 0) {
			const effectiveContainerWidth = getEffectiveWidth(currentRowImages.length);
			let scale = effectiveContainerWidth / currentRowWidthSum;

			const processedImages: ImageFile[] = currentRowImages.map((img) => {
				const imgAspectRatio = img.width && img.height ? img.width / img.height : 16 / 9;
				return {
					width: Math.floor(targetRowHeight * imgAspectRatio * scale),
					height: Math.floor(targetRowHeight * scale),
					id: img.id
				};
			});

			rows.push({
				files: processedImages,
				height: processedImages[0]!.height,
				index: rows.length
			});
		}

		console.log("created:", rows.length)
		return rows;
	}
</script>

<div bind:this={element} class="w-full" style="min-height: {MIN_HEIGHT_PX}px;">
	{#if isVisible && hasFetched}
		<h1 class="mb-2 text-3xl">
			{date.toLocaleDateString('en-UK', { month: 'long', day: 'numeric', year: 'numeric' })}
		</h1>
		{#if allFiles.length > 0}
			<Virtlist rows={buildMasonry()}></Virtlist>

			<!-- <div id="gridContainer" class="grid grid-cols-6 gap-2"> -->
			<!-- {#each renderedFiles as f (f.id)}
					<div class="aspect-square overflow-hidden bg-zinc-800">
						<img
							src="{PUBLIC_SERVER_URL}/api/files/{f.id}/view"
							alt="Image {f.id}"
							class="h-full w-full object-cover"
							loading="lazy"
							decoding="async"
						/>
					</div>
				{/each} -->
			<!-- </div> -->
		{:else if !isRendering}
			<p class="text-center text-gray-500">No images found for this date.</p>
		{/if}
		<hr class="my-4" />
	{:else}
		<div style="height: {getPlaceholderHeight()}px;"></div>
	{/if}
</div>
