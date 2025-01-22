<script lang="ts">
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { deleteVault, type Vault } from '@/api/vaults';
	import { buttonVariants } from '@/components/ui/button';
	import Input from '@/components/ui/input/input.svelte';
	import Label from '@/components/ui/label/label.svelte';
	import { activeVault, vaultStore } from '@/stores';
	import { cn } from '@/utils';
	import { Trash } from 'lucide-svelte';
	import { get } from 'svelte/store';

	const vault: Vault = $props();
	let inputVaultName = $state('');
	let dialogOpen = $state(false);
	let buttonEnabled = $derived(vault.name.trim() === inputVaultName.trim());

	async function onButtonClick() {
		await deleteVault({ id: vault.id });
		vaultStore.update((v) => {
			v = v.filter((x) => x.id !== vault.id);
			return v;
		});

		if (get(activeVault)?.id === vault.id) {
			activeVault.set(undefined);
		}

		dialogOpen = false;
	}

	function onkeyup(ev: KeyboardEvent & { currentTarget: EventTarget & Window }) {
		ev.stopPropagation();
		ev.preventDefault();
		if (ev.code === 'Enter' && buttonEnabled) {
			onButtonClick();
		}
	}
</script>

<svelte:window {onkeyup} />

<AlertDialog.Root bind:open={dialogOpen}>
	<AlertDialog.Trigger
		onclick={() => {
			dialogOpen = !dialogOpen;
		}}
	>
		<button
			class="bg-background border-border flex h-10 w-10 flex-row items-center justify-center rounded-md border"
		>
			<Trash class="h-5"></Trash>
		</button>
	</AlertDialog.Trigger>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Are you sure you want to delete {vault.name}</AlertDialog.Title>
			<AlertDialog.Description>
				This action cannot be undone. This will permanently delete the vault and all of its files.
				Enter the vault name to confirm
			</AlertDialog.Description>
		</AlertDialog.Header>

		<div class="grid gap-1.5">
			<Label>To confirm, type '{vault.name}' below</Label>
			<Input bind:value={inputVaultName} />
		</div>

		<AlertDialog.Footer>
			<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action
				class={cn(buttonVariants({ variant: 'destructive' }))}
				onclick={onButtonClick}
				disabled={!buttonEnabled}
			>
				Continue
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
