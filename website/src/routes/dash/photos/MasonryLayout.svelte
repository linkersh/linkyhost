<script lang="ts">
	import type { APIFile } from '$lib/api/files';
	import { PUBLIC_SERVER_URL } from '$env/static/public';
	import { onDestroy, onMount } from 'svelte';
	import type { BucketWithFiles } from '$lib/gallery';
	import type { Writable } from 'svelte/store';

	type FullAPIFile = APIFile & { _width: number; _height: number };

	interface ImageRow {
		images: FullAPIFile[];
	}

	interface GroupedImageRow {
		dateLabel: string;
		bucketId: string;
		element?: HTMLDivElement;
		rows: ImageRow[];
	}

	interface Props {
		buckets: BucketWithFiles[];
		scrollBucketStore: Writable<string | undefined>;
	}

	let { buckets = $bindable(), scrollBucketStore }: Props = $props();
	let container: HTMLDivElement;
	let groupedImageRows: GroupedImageRow[] = $state([]);
	let resizeObserver: ResizeObserver;

	const targetRowHeight = 320;
	const maxRowHeight = 400;
	const imageGap = 4;
	const rowGap = 4;
	const minImagesPerRow = 2;
	const maxImagesPerRow = 5;
	const maxSingleImageAspectRatio = 1.8;

	function getStartOfDay(date: Date): Date {
		const newDate = new Date(date);
		newDate.setHours(0, 0, 0, 0);
		return newDate;
	}

	function formatDateLabel(date: Date): string {
		const today = getStartOfDay(new Date());
		const yesterday = new Date(today);
		yesterday.setDate(today.getDate() - 1);
		const oneWeekAgo = new Date(today);
		oneWeekAgo.setDate(today.getDate() - 7);

		const inputDay = getStartOfDay(date);

		if (inputDay.getTime() === today.getTime()) {
			return 'Today';
		}
		if (inputDay.getTime() === yesterday.getTime()) {
			return 'Yesterday';
		}
		if (inputDay > oneWeekAgo) {
			return inputDay.toLocaleDateString(undefined, { weekday: 'long' });
		}
		if (inputDay.getFullYear() === today.getFullYear()) {
			return inputDay.toLocaleDateString(undefined, { month: 'long', day: 'numeric' });
		}
		return inputDay.toLocaleDateString(undefined, {
			year: 'numeric',
			month: 'long',
			day: 'numeric'
		});
	}

	function calculateLayout() {
		if (!container || !buckets || buckets.length === 0) {
			groupedImageRows = [];
			return;
		}

		const containerWidth = container.clientWidth;
		if (containerWidth <= 0) return;

		const imagesByDate = new Map<string, BucketWithFiles>();

		for (const bucket of buckets) {
			for (const img of bucket.files) {
				const createdAtDate = new Date(img.createdAt);
				if (isNaN(createdAtDate.getTime())) {
					console.warn('Invalid date found for image:', img.id);
					const dateKey = 'unknown';
					if (!imagesByDate.has(dateKey)) {
						imagesByDate.set(dateKey, { bucketId: bucket.bucketId, files: [] });
					}
					imagesByDate.get(dateKey)!.files.push(img);
					continue;
				}

				const dateKey = getStartOfDay(createdAtDate).toISOString();
				if (!imagesByDate.has(dateKey)) {
					imagesByDate.set(dateKey, { bucketId: bucket.bucketId, files: [] });
				}
				imagesByDate.get(dateKey)!.files.push(img);
			}
		}

		console.log('---IMAGES BY DATE', imagesByDate);

		const newGroupedRows: GroupedImageRow[] = [];
		const dateKeys = Array.from(imagesByDate.keys());

		for (const dateKey of dateKeys) {
			const groupImages = imagesByDate.get(dateKey)!;
			const groupRows: ImageRow[] = [];
			let currentRowImages: APIFile[] = [];
			let currentRowWidthSum = 0;

			const getEffectiveWidth = (numImages: number) => {
				const gaps = Math.max(0, numImages - 1) * imageGap;
				return containerWidth - gaps;
			};

			for (let i = 0; i < groupImages.files.length; i++) {
				const img = groupImages.files[i];
				const aspectRatio = img.width && img.height ? img.width / img.height : 16 / 9;
				const scaledWidth = targetRowHeight * aspectRatio;

				currentRowImages.push(img);
				currentRowWidthSum += scaledWidth;

				const isLastImageInGroup = i === groupImages.files.length - 1;
				const hasMinImagesInRow = currentRowImages.length >= minImagesPerRow;
				const hasMaxImagesInRow = currentRowImages.length >= maxImagesPerRow;
				const effectiveContainerWidth = getEffectiveWidth(currentRowImages.length);
				const wouldExceedWidth = currentRowWidthSum > effectiveContainerWidth;

				if ((hasMinImagesInRow && wouldExceedWidth) || hasMaxImagesInRow || isLastImageInGroup) {
					let scale = effectiveContainerWidth / currentRowWidthSum;

					if (currentRowImages.length === 1) {
						const singleImage = currentRowImages[0];
						const singleAspectRatio =
							singleImage.width && singleImage.height
								? singleImage.width / singleImage.height
								: 16 / 9;
						const maxWidth = containerWidth;
						const desiredWidth = targetRowHeight * singleAspectRatio;
						let finalWidth = Math.min(maxWidth, desiredWidth * scale);

						const currentAspectRatio = finalWidth / (targetRowHeight * scale);
						if (currentAspectRatio > maxSingleImageAspectRatio && finalWidth === maxWidth) {
							finalWidth = targetRowHeight * scale * maxSingleImageAspectRatio;
							scale = finalWidth / desiredWidth;
						} else if (finalWidth < maxWidth && desiredWidth * scale > maxWidth) {
							scale = maxWidth / desiredWidth;
						}

						let resultingHeight = targetRowHeight * scale;
						if (resultingHeight > maxRowHeight) {
							scale *= maxRowHeight / resultingHeight;
						}
					} else {
						let resultingHeight = targetRowHeight * scale;
						if (resultingHeight > maxRowHeight) {
							scale = maxRowHeight / targetRowHeight;
						}
					}

					const processedImages = currentRowImages.map((img) => {
						const imgAspectRatio = img.width && img.height ? img.width / img.height : 16 / 9;
						return {
							...img,
							_width: targetRowHeight * imgAspectRatio * scale,
							_height: targetRowHeight * scale
						};
					});

					groupRows.push({ images: processedImages });
					currentRowImages = [];
					currentRowWidthSum = 0;
				}
			}

			const representativeDate = dateKey === 'unknown' ? null : new Date(dateKey);
			const dateLabel = representativeDate ? formatDateLabel(representativeDate) : 'Unknown Date';
			newGroupedRows.push({
				dateLabel: dateLabel,
				bucketId: groupImages.bucketId,
				rows: groupRows
			});
		}

		groupedImageRows = newGroupedRows;
	}

	$effect(() => {
		if (buckets && container) {
			calculateLayout();
		}
	});

	function scrollToBucket(bucketId: string) {
		console.log(groupedImageRows);
		const group = groupedImageRows.find((g) => g.bucketId === bucketId);
		if (group?.element) {
			group.element.scrollIntoView({ behavior: 'smooth', block: 'start' });
		}
	}

	let scrollSub: () => void | undefined;

	onMount(async () => {
		if (container) {
			calculateLayout();
		}
		resizeObserver = new ResizeObserver(() => {
			calculateLayout();
		});
		resizeObserver.observe(container);
		scrollSub = scrollBucketStore.subscribe((bucketId) => {
			if (bucketId) {
				scrollToBucket(bucketId);
			}
		});
	});

	onDestroy(() => {
		resizeObserver?.disconnect();
		scrollSub?.();
	});

	function renderGroups() {
		if (!container) return;

		// Clear existing content
		container.innerHTML = '';

		groupedImageRows.forEach((group) => {
			const groupDiv = document.createElement('div');
			groupDiv.className = 'mb-3 mt-5';
			group.element = groupDiv;

			const heading = document.createElement('h2');
			heading.className = 'mb-2 text-2xl font-medium text-white';
			heading.textContent = group.dateLabel;
			groupDiv.appendChild(heading);

			group.rows.forEach((row) => {
				const rowDiv = document.createElement('div');
				rowDiv.className = 'flex w-full flex-row';
				rowDiv.style.fontSize = '0';
				rowDiv.style.lineHeight = '0';
				rowDiv.style.marginBottom = `${rowGap}px`;

				row.images.forEach((image, imageIndex) => {
					const imageContainer = document.createElement('div');
					imageContainer.style.width = `${image._width}px`;
					imageContainer.style.height = `${image._height}px`;
					imageContainer.style.display = 'inline-block';
					imageContainer.style.overflow = 'hidden';
					if (imageIndex < row.images.length - 1) {
						imageContainer.style.marginRight = `${imageGap}px`;
					}
					imageContainer.className = 'rounded-lg bg-zinc-700 shadow-sm';

					const img = document.createElement('img');
					img.src = `${PUBLIC_SERVER_URL}/api/files/${image.id}/view`;
					img.style.width = '100%';
					img.style.height = '100%';
					img.style.objectFit = 'cover';
					img.alt = image.fileName;
					img.loading = 'lazy';

					imageContainer.appendChild(img);
					rowDiv.appendChild(imageContainer);
				});

				groupDiv.appendChild(rowDiv);
			});

			container.appendChild(groupDiv);
		});
	}

	$effect(() => {
		if (groupedImageRows && container) {
			renderGroups();
		}
	});
</script>

<!-- svelte-ignore element_invalid_self_closing_tag -->
<div bind:this={container} class="w-[95%]" />
