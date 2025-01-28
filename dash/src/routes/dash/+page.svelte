<script lang="ts">
	import * as Select from '$lib/components/ui/select/index.js';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import type { VaultFile } from '@/api/vaults';
	import { activeVault, refreshActiveVaultFiles } from '@/stores';
	import { onDestroy, onMount } from 'svelte';
	import { buttonVariants } from '@/components/ui/button';
	import { cn } from '@/utils';
	import { get } from 'svelte/store';
	import Input from '@/components/ui/input/input.svelte';
	import FileThumbnail from './FileThumbnail.svelte';
	import DeleteVaultDialog from './DeleteVaultDialog.svelte';
	import UploadDialog from '@/components/upload-dialog/UploadDialog.svelte';
	import NoThumbnailFile from './NoThumbnailFile.svelte';
	import Label from '@/components/ui/label/label.svelte';

	let vaultPassword = $state('');
	let vaultPwdSubmitEnabled = $derived(vaultPassword.length > 8);

	let isUnlocked = $state(false);
	let pages = $state<Array<VaultFile[]>>([]);
	let isIntersecting = false;
	let activeVaultSub: () => void | undefined;

	const limit = 100;
	let skip = 0;

	onMount(async () => {
		activeVaultSub = activeVault.subscribe(async (v) => {
			if (!v) return;

			console.log('select vault:', v);
			isUnlocked = false;
			vaultPassword = '';

			if (v.is_encrypted) {
				return;
			}

			pages = [await refreshActiveVaultFiles({ vaultId: v.id, limit, skip })];

			setTimeout(() => {
				const observer = new IntersectionObserver(
					async (entries) => {
						const wasIntersecting = isIntersecting;

						isIntersecting = entries[0].isIntersecting;
						if (!wasIntersecting && isIntersecting) {
							console.log('reached bottom of page, loading more');
							skip += limit;
							pages.push(await refreshActiveVaultFiles({ vaultId: v.id, limit, skip }));
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
	});

	onDestroy(() => {
		activeVaultSub?.();
	});

	async function unlockVault() {
		isUnlocked = true;

		const vault = get(activeVault)!;
		sessionStorage.setItem(`vault_${vault.id}_password`, vaultPassword);

		console.log('unlocked vault:', vault.name);
		pages = [await refreshActiveVaultFiles({ vaultId: vault.id, limit, skip })];
		console.log(`fetched ${pages[0].length} files for vault: ${vault.name}`);
	}
</script>

{#if $activeVault}
	{#if isUnlocked || !$activeVault.is_encrypted}
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
	{:else}
		<AlertDialog.Root open={true}>
			<AlertDialog.Content>
				<AlertDialog.Header>
					<AlertDialog.Title>Input password to decrypt vault.</AlertDialog.Title>
					<AlertDialog.Description>
						The password will be stored in your session storage as long as this website tab is open.
					</AlertDialog.Description>
				</AlertDialog.Header>

				<div class="grid gap-1.5">
					<Label>Vault password</Label>
					<Input bind:value={vaultPassword} />
				</div>

				<AlertDialog.Footer>
					<AlertDialog.Cancel
						onclick={() => {
							activeVault.set(undefined);
						}}>Cancel</AlertDialog.Cancel
					>
					<AlertDialog.Action
						class={cn(buttonVariants({ variant: 'destructive' }))}
						onclick={unlockVault}
						disabled={!vaultPwdSubmitEnabled}
					>
						Unlock
					</AlertDialog.Action>
				</AlertDialog.Footer>
			</AlertDialog.Content>
		</AlertDialog.Root>
	{/if}
{/if}
