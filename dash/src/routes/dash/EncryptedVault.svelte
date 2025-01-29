<script lang="ts">
	import { downloadFile, getCheckFile, type Vault } from '@/api/vaults';
	import { KeyRound } from 'lucide-svelte';
	import * as Card from '$lib/components/ui/card/index';
	import Label from '@/components/ui/label/label.svelte';
	import Input from '@/components/ui/input/input.svelte';
	import Button from '@/components/ui/button/button.svelte';
	import { decryptFile } from '@/encryption';

	const vault: Vault = $props();

	let vaultPassword = $state('');
	let passwordInvalid = $state(false);
	let isVaultUnlocked = $state(false);
	let isUnlockEnabled = $derived(vaultPassword.length > 8);

	$effect(() => {
		vaultPassword;
		passwordInvalid = false;
	});

	async function unlockVault() {
		passwordInvalid = false;
		try {
			const checkFile = await getCheckFile(vault.id);
			const checkFileData = await downloadFile({
				vaultId: checkFile.vault_id,
				fileId: checkFile.id
			});

			await decryptFile(
				checkFileData,
				Uint8Array.from(checkFile.password_salt),
				Uint8Array.from(checkFile.fixed_iv),
				checkFile.chunk_size,
				vaultPassword
			);

			console.log('vault decrypted successfully');
		} catch (err) {
			passwordInvalid = true;
			console.error('failed to decrypt vault', err);
		}
	}
</script>

{#if isVaultUnlocked}
	<div></div>
{:else}
	<div class="flex h-full w-full flex-row items-center justify-center">
		<Card.Root>
			<Card.Header>
				<Card.Title>Input vault password to decrypt files</Card.Title>
				<Card.Description>Decryption is done only on the client-side</Card.Description>
			</Card.Header>

			<Card.Content>
				<div class="grid gap-1.5">
					<Label>Password</Label>
					<Input type="password" bind:value={vaultPassword} />
				</div>

				{#if passwordInvalid}
					<p class="text-red-500">Password is invalid</p>
				{/if}

				<Button
					onclick={unlockVault}
					disabled={!isUnlockEnabled}
					class="mt-2 flex w-full flex-row items-center gap-2"
				>
					<span>Unlock vault</span>
					<KeyRound></KeyRound>
				</Button>
			</Card.Content>
		</Card.Root>
	</div>
{/if}
