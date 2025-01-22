<script lang="ts">
	import Progress from '@/components/ui/progress/progress.svelte';
	import { vaultStore } from '@/stores';
	import { uploader, type ActiveUpload } from '@/uploads';
	import { X } from 'lucide-svelte';
	import { cubicOut } from 'svelte/easing';
	import { get } from 'svelte/store';
	import { scale } from 'svelte/transition';

	const upload: ActiveUpload = $props();
	function findVault() {
		const vaults = get(vaultStore);
		const vault = vaults.find((x) => x.id === upload.vaultId);
		return vault?.name ?? 'unknown vault';
	}

	function cancelUpload() {
		uploader.cancelUpload(upload.id);
	}
</script>

<div
	transition:scale={{ easing: cubicOut, duration: 200 }}
	class="bg-background border-border flex w-96 flex-col gap-2 rounded-lg border p-4"
>
	{#if upload.status === 'uploading'}
		<div class="flex flex-row justify-between">
			<div class="flex flex-col">
				<span class="text-lg font-semibold">Vault {findVault()}</span>
				<span>Uploading {upload.currentFileName}</span>
			</div>

			<button onclick={cancelUpload}>
				<X class="h-5"></X>
			</button>
		</div>

		<div class="flex flex-row items-center gap-2">
			<Progress max={100} value={upload.totalProgress} class="w-full"></Progress>
			<span>{Math.min(Math.round(upload.totalProgress), 100)}%</span>
		</div>
	{:else if upload.status === 'cancelled'}
		<div class="flex flex-col">
			<span class="text-lg font-semibold">Vault {findVault()}</span>
			<span>Upload cancelled</span>
		</div>
	{:else if upload.status === 'completed'}
		<div class="flex flex-col">
			<span class="text-lg font-semibold">Vault {findVault()}</span>
			<span>Upload completed</span>
		</div>
	{/if}
</div>
