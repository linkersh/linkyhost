<script lang="ts">
	import * as Select from '$lib/components/ui/select/index.js';
	import { fetchFiles, type VaultFile } from '@/api/vaults';
	import { activeVault } from '@/stores';
	import { onDestroy, onMount } from 'svelte';
	import Input from '@/components/ui/input/input.svelte';
	import FileThumbnail from './FileThumbnail.svelte';
	import DeleteVaultDialog from './DeleteVaultDialog.svelte';
	import UploadDialog from '@/components/upload-dialog/UploadDialog.svelte';

	const limit = 100;
	let files = $state<VaultFile[]>([]);
	let isLoaded = $state(false);
	let skip = 0;
	let activeVaultSub: () => void | undefined;

	onMount(() => {
		activeVaultSub = activeVault.subscribe(async (v) => {
			if (!v) return;
			files = await fetchFiles({ limit, skip, vaultId: v.id });
			isLoaded = true;
		});
	});

	onDestroy(() => {
		activeVaultSub?.();
		files = [];
	});
</script>

{#if $activeVault}
	<div class="flex items-center justify-between">
		<h1 class="text-4xl font-medium">{$activeVault.name}</h1>

		<div class="flex items-center gap-2">
			<UploadDialog vaultId={$activeVault.id}></UploadDialog>
			<DeleteVaultDialog {...$activeVault}></DeleteVaultDialog>
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

	<div class="mt-6 grid grid-cols-6 gap-2">
		{#if isLoaded}
			{#each files as file}
				{#if file.content_type.startsWith('image/')}
					<FileThumbnail {...file}></FileThumbnail>
				{/if}
			{/each}

		{:else}
			<p>loading</p>
		{/if}
	</div>
{/if}
