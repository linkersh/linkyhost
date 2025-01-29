<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index';
	import Input from './ui/input/input.svelte';
	import Label from './ui/label/label.svelte';
	import Switch from './ui/switch/switch.svelte';
	import Button from './ui/button/button.svelte';
	import { Plus } from 'lucide-svelte';
	import { createVault } from '@/api/vaults';
	import { activeVault, vaultStore } from '@/stores';
	import type { GroupUploadMeta } from '@/uploads';
	import { encryptFile } from '@/encryption';

	let vaultName = $state('');
	let password = $state('');
	let enableOCR = $state(false);
	let enableEncrypt = $state(false);
	let createBtnEnable = $derived(
		vaultName.length > 2 && (enableEncrypt ? password.length > 8 : true)
	);
	let dialogOpen = $state(false);

	async function createBtnClick() {
		const vault = await createVault({ name: vaultName, is_encrypted: enableEncrypt });

		if (enableEncrypt) {
			const payload = { version: 1, vaultId: vault.id };
			const blob = new Blob([JSON.stringify(payload)], { type: 'application/json' });

			// We need to create a check file
			// A check file is used to verify whether a password is valid when decrypting a vault.

			const encrypted = await encryptFile(blob, password);
			console.log('[GROUP UPLOAD] attempting to upload vault check file');

			const file = new File([encrypted.encryptedData], '__vault_check_file', {
				type: 'application/json'
			});

			const formData = new FormData();
			const url = `http://127.0.0.1:8080/api/vaults/${vault.id}/groupUpload`;
			const metadata: GroupUploadMeta = {
				chunk_size: encrypted.chunkSize,
				content_type: blob.type,
				file_name: file.name,
				file_size: encrypted.encryptedData.size,
				is_encrypted: true,
				is_hidden: true,
				fixed_iv: Array.from(encrypted.fixedIv),
				salt: Array.from(encrypted.salt)
			};

			formData.append('data', file);
			formData.append('metadata', JSON.stringify([metadata]));

			const req = await fetch(url, {
				method: 'POST',
				body: formData,
				headers: {
					Authorization: localStorage.getItem('token')!
				}
			});
			if (req.status >= 400) {
				console.error(`failed to create vault check file: ${req.statusText}`);
			}
		}

		vaultStore.update((x) => {
			x.push(vault);
			return x;
		});

		activeVault.set(vault);

		// reset all props incase user creates more vaults
		vaultName = '';
		enableOCR = false;
		enableEncrypt = false;
		dialogOpen = false;
	}
</script>

<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Trigger
		onclick={(o) => {
			dialogOpen = !dialogOpen;
			o.stopPropagation();
		}}
	>
		<Sidebar.MenuAction>
			<Plus />
		</Sidebar.MenuAction>
	</Dialog.Trigger>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Create new Vault</Dialog.Title>
		</Dialog.Header>

		<div class="mt-4 flex flex-col gap-2">
			<div class="grid gap-1.5">
				<Label>Vault name</Label>
				<Input bind:value={vaultName} />
			</div>

			<div class="flex items-center justify-between">
				<Label>Enable OCR</Label>
				<Switch bind:checked={enableOCR}></Switch>
			</div>

			<div class="flex items-center justify-between">
				<Label>Encrypt</Label>
				<Switch bind:checked={enableEncrypt}></Switch>
			</div>

			{#if enableEncrypt}
				<div class="my-2 grid gap-1.5">
					<Label for="password">Choose a password</Label>
					<Input bind:value={password} type="password" />
					<p class="text-muted-foreground text-sm">
						Make sure to remember this password or write it down somewhere As you wont be able to
						ever recover this data without the correct password. Min. 8 characters
					</p>
				</div>
			{/if}

			<Button onclick={createBtnClick} disabled={!createBtnEnable}>Create Vault</Button>
		</div>
	</Dialog.Content>
</Dialog.Root>
