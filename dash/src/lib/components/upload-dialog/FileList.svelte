<script lang="ts">
	import { X } from 'lucide-svelte';

	let { files = $bindable() }: { files: File[] } = $props();

	function removeFile(idx: number) {
		files = files.filter((_, i) => i !== idx);
	}
</script>

{#if files.length > 0}
	<div class="flex max-h-56 w-full flex-col gap-2 overflow-y-auto p-2">
		{#each files as file, i}
			<div class="flex flex-row items-center justify-between">
				<span>{file.name}</span>
				<button onclick={() => removeFile(i)}>
					<X></X>
				</button>
			</div>
		{/each}
	</div>
{/if}
