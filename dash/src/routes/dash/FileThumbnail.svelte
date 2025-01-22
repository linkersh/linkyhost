<script lang="ts">
	import type { VaultFile } from '@/api/vaults';
	import { onDestroy, onMount } from 'svelte';

	const file: VaultFile = $props();
	const url = `http://localhost:8080/api/vaults/${file.vault_id}/files/${file.id}/thumbnail`;
	let ourl = $state<string | undefined>();

	onMount(async () => {
		const req = await fetch(url, {
			headers: { Authorization: `${localStorage.getItem('token')}` }
		});
		const body = await req.blob();
		ourl = URL.createObjectURL(body);
	});

	onDestroy(() => {
		if (ourl) {
			URL.revokeObjectURL(ourl);
		}
	});
</script>

{#if ourl}
	<img alt="skibidi toilet" class="w-full h-56 object-cover rounded-lg" src={ourl} />
{/if}
