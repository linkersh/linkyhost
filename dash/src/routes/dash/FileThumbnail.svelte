<script lang="ts">
	import { downloadFile, type VaultFile } from '@/api/vaults';
	import { decryptFile } from '@/encryption';
	import { onDestroy, onMount } from 'svelte';

	const file: VaultFile = $props();
	const url = `http://localhost:8080/api/vaults/${file.vault_id}/files/${file.id}/thumbnail`;
	let ourl = $state<string | undefined>();

	onMount(async () => {
		if (file.is_encrypted) {
			const blob = await downloadFile({ vaultId: file.vault_id, fileId: file.id });
			const salt = Uint8Array.from(file.password_salt);
			const fixedIv = Uint8Array.from(file.fixed_iv);
			const data = await decryptFile(
				blob,
				salt,
				fixedIv,
				file.chunk_size,
				sessionStorage.getItem(`vault_${file.vault_id}_password`)!
			);

			const url = URL.createObjectURL(data);
			ourl = url;
		} else {
			const req = await fetch(url, {
				headers: { Authorization: `${localStorage.getItem('token')}` }
			});
			const body = await req.blob();
			ourl = URL.createObjectURL(body);
		}
	});

	onDestroy(() => {
		if (ourl) {
			URL.revokeObjectURL(ourl);
		}
	});
</script>

{#if ourl}
	<img
		alt={file.file_name}
		class="imageGridItem m-2 h-auto w-full rounded-lg object-cover"
		src={ourl}
	/>
{/if}

<style>
	img {
		max-width: 100%;
		display: block;
	}
</style>
