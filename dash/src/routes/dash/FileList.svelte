<script lang="ts">
	import * as Select from '$lib/components/ui/select/index.js';

	import { fetchFiles, type Vault, type VaultFile } from '@/api/vaults';
	import { onDestroy, onMount } from 'svelte';
	import { Input } from '@/components/ui/input';

	import FileThumbnail from './FileThumbnail.svelte';
	import NoThumbnailFile from './NoThumbnailFile.svelte';
	import UploadDialog from '@/components/upload-dialog/UploadDialog.svelte';
	import DeleteVaultDialog from './DeleteVaultDialog.svelte';

	interface Props {
		vault: Vault;
	}

	let limit = 100;
	let skip = 0;
	let isIntersecting = false;
	let observer: IntersectionObserver | undefined;

	let pages = $state<Array<VaultFile[]>>([]);

	let { vault }: Props = $props();
	let vaultId = vault.id;

	onMount(async () => {
		pages = [await fetchFiles({ vaultId, limit, skip })];

		setTimeout(() => {
			observer = new IntersectionObserver(
				async (entries) => {
					const wasIntersecting = isIntersecting;

					isIntersecting = entries[0].isIntersecting;
					if (!wasIntersecting && isIntersecting) {
						console.log('reached bottom of page, loading more');
						const nextPage = await fetchFiles({ vaultId, limit, skip: skip + limit });
						if (nextPage.length > 0) {
							skip += limit;
						}

						pages.push(nextPage); // maybe wont work
					}
				},
				{
					root: null,
					rootMargin: '100px',
					threshold: 0.1
				}
			);

			const sentinel = document.getElementById('sentinel')!;
			observer.observe(sentinel);
		}, 100);
	});

	onDestroy(() => {
		if (observer) {
			observer.disconnect();
		}
	});
</script>

<div class="flex items-center justify-between">
	<h1 class="text-4xl font-medium">{vault.name}</h1>

	<div class="flex items-center gap-2">
		<UploadDialog {...vault}></UploadDialog>
		<DeleteVaultDialog {...vault}></DeleteVaultDialog>
	</div>
</div>

<div class="mt-4 flex flex-row items-center gap-2">
	<Input placeholder="Search a file.." />

	<Select.Root value="sortNew" type="single">
		<Select.Trigger class="w-[180px]">Order by</Select.Trigger>
		<Select.Content>
			<Select.Item value="sortNew">Newest</Select.Item>
			<Select.Item value="sortOld">Oldest</Select.Item>
			<Select.Item value="sortAZ">A-Z</Select.Item>
			<Select.Item value="sortZA">Z-A</Select.Item>
		</Select.Content>
	</Select.Root>
</div>

<div id="imageGrid">
	{#each pages as page}
		<div class="mt-6 w-full columns-5">
			{#each page as file}
				{#if file.content_type.startsWith('image/')}
					<FileThumbnail {...file}></FileThumbnail>
				{:else}
					<NoThumbnailFile {...file}></NoThumbnailFile>
				{/if}
			{/each}
		</div>
	{/each}
	<div id="sentinel" class="h-4 w-full"></div>
</div>
