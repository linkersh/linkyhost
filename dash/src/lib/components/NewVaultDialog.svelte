<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index';
	import Input from './ui/input/input.svelte';
	import Label from './ui/label/label.svelte';
	import Switch from './ui/switch/switch.svelte';
	import Button from './ui/button/button.svelte';
	import { Plus } from 'lucide-svelte';
	import { createVault, VaultFlags } from '@/api/vaults';
	import { activeVault, vaultStore } from '@/stores';

	let vaultName = $state('');
	let enableOCR = $state(false);
	let enableEncrypt = $state(false);
	let createBtnEnable = $derived(vaultName.length > 2);
	let dialogOpen = $state(false);

	async function createBtnClick() {
		let flags = 0;
		if (enableOCR) {
			flags |= VaultFlags.OCR;
		}
		if (enableEncrypt) {
			flags |= VaultFlags.Encrypted;
		}
		const vault = await createVault({ name: vaultName, flags });
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
			<Button onclick={createBtnClick} disabled={!createBtnEnable}>Create Vault</Button>
		</div>
	</Dialog.Content>
</Dialog.Root>
