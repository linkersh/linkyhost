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
	import EncryptedVault from './EncryptedVault.svelte';

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

{#if $activeVault && $activeVault.is_encrypted}
	<EncryptedVault {...$activeVault}></EncryptedVault>
{/if}
