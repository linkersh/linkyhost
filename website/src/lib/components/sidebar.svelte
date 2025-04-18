<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import { page } from '$app/state';
	import { Image, Music2, Plus, Search, Settings } from '@lucide/svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import UploadDialog from './upload-dialog.svelte';

	const pages = [
		{
			name: 'Photos',
			icon: Image,
			link: '/dash/photos'
		},
		{
			name: 'Audios',
			icon: Music2,
			link: '/dash/audios'
		},
		{
			name: 'Settings',
			icon: Settings,
			link: '/dash/settings'
		}
	];

	let uploadDialogOpen = $state(false);

	afterNavigate(() => {
		const url = page.url.href;
		console.log(url);
	});
</script>

<UploadDialog bind:open={uploadDialogOpen}></UploadDialog>

<div class="flex h-full w-full flex-col border-r px-2 pt-2">
	<div class="mb-2 flex flex-row items-center gap-2">
		<Button
			onclick={() => {
				uploadDialogOpen = !uploadDialogOpen;
			}}
			class="w-full rounded-xl"
			variant="outline"
			aria-label="Add"
		>
			<Plus size="16" aria-hidden="true"></Plus>
		</Button>
		<Button class="w-full rounded-xl" variant="outline" aria-label="Search">
			<Search size="16" aria-hidden="true"></Search>
		</Button>
	</div>

	{#each pages as p}
		<a
			href={p.link}
			class="flex h-10 items-center gap-2 rounded-[var(--radius)] p-2 tracking-tight transition-colors {page.url.href.endsWith(
				p.link
			)
				? 'bg-secondary drop-shadow-md'
				: ''} mb-1"
		>
			<p.icon size="16"></p.icon>
			<span>{p.name}</span>
		</a>
	{/each}
</div>
