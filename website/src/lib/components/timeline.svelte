<script lang="ts">
	import { getGallery } from '$lib/gallery';
	import { onMount } from 'svelte';

	interface YearData {
		year: number;
		date: string;
		headerElement: HTMLDivElement;
		parentElement: HTMLDivElement;
		months: Array<{ date: string; month: number; element: HTMLDivElement; count: number }>;
	}

	let years: YearData[] = $state([]);
	let cursor: HTMLDivElement;
	let container: HTMLDivElement;
	let isDragging = false;

	let scheduledDate: string | null = null;
	let timeout: NodeJS.Timeout | null = null;

	function scheduleFetch(date: string, count: number) {
		if (scheduledDate === date) return;
		if (timeout) {
			clearTimeout(timeout);
		}

		timeout = setTimeout(async () => {
			const gallery = await getGallery();
			gallery.setCurrentBucket({ count, date });
		}, 500);
	}

	function checkForOverlap(el1: HTMLDivElement, el2: HTMLDivElement) {
		const bounds1 = el1.getBoundingClientRect();
		const bounds2 = el2.getBoundingClientRect();

		const isFirstLeftmost = bounds1.left <= bounds2.left;
		const leftElement = isFirstLeftmost ? bounds1 : bounds2;
		const rightElement = isFirstLeftmost ? bounds2 : bounds1;

		if (leftElement.right > rightElement.left) {
			const isFirstTopmost = bounds1.top <= bounds2.top;
			const topElement = isFirstTopmost ? bounds1 : bounds2;
			const bottomElement = isFirstTopmost ? bounds2 : bounds1;

			return topElement.bottom > bottomElement.top;
		}

		return false;
	}

	function updateCursorPosition(clientY: number) {
		const containerRect = container.getBoundingClientRect();
		const relativeY = Math.min(Math.max(clientY - containerRect.top, 0), containerRect.height);
		cursor.style.top = `${relativeY}px`;

		for (const year of years) {
			if (checkForOverlap(year.headerElement, cursor)) {
				scheduleFetch(year.date, year.months[0].count);
				return;
			}

			const month = year.months.find((x) => checkForOverlap(x.element, cursor));
			if (month) {
				scheduleFetch(month.date, month.count);
				return;
			}
		}
	}

	function onMouseDown(e: MouseEvent) {
		e.preventDefault();
		isDragging = true;
		updateCursorPosition(e.clientY);
	}

	function onMouseMove(e: MouseEvent) {
		if (!isDragging) return;
		e.preventDefault();
		updateCursorPosition(e.clientY);
	}

	function onMouseUp() {
		isDragging = false;
	}

	onMount(async () => {
		for (const bucket of (await getGallery()).buckets) {
			const date = new Date(bucket.date);
			const year = years.find((x) => x.year === date.getUTCFullYear());
			if (!year) {
				const h1 = document.createElement('h1');
				h1.textContent = date.getUTCFullYear().toString();
				h1.className = 'select-none text-white text-sm';

				container.appendChild(h1);

				const div = document.createElement('div');
				div.className = 'flex flex-col items-center gap-2';

				const monthDiv = document.createElement('div');
				monthDiv.className = 'h-1 w-1 rounded-full bg-white';

				div.appendChild(monthDiv);
				container.appendChild(div);

				years = [
					...years,
					{
						date: bucket.date,
						year: date.getUTCFullYear(),
						months: [
							{
								date: bucket.date,
								element: monthDiv,
								month: date.getUTCMonth() + 1,
								count: bucket.count
							}
						],
						headerElement: h1,
						parentElement: div
					}
				];
			} else {
				const monthDiv = document.createElement('div');
				monthDiv.className = 'h-1 w-1 rounded-full bg-white';
				year.parentElement.appendChild(monthDiv);

				year.months.push({
					date: bucket.date,
					month: date.getUTCMonth() + 1,
					element: monthDiv,
					count: bucket.count
				});
			}
		}
	});
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	onmousedown={onMouseDown}
	onmouseup={onMouseUp}
	onmousemove={onMouseMove}
	bind:this={container}
	class="relative flex h-full w-16 select-none flex-col items-center gap-1 hover:cursor-n-resize"
>
	<div bind:this={cursor} class="absolute z-10 h-1 w-full bg-blue-300"></div>
</div>
