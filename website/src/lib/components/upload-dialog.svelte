<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { enqueueUpload } from '$lib/queue';
	import Button from './ui/button/button.svelte';
	import { X } from '@lucide/svelte';

	let { open = $bindable() } = $props();

	let isDragging = $state(false);
	let fileInput: HTMLInputElement;
	let selectedFiles: File[] = $state([]);

	function handleDragOver(event: DragEvent) {
		event.preventDefault();
		isDragging = true;
	}

	function handleDragLeave(event: DragEvent) {
		event.preventDefault();
		isDragging = false;
	}

	function handleDrop(event: DragEvent) {
		event.preventDefault();
		isDragging = false;
		const files = event.dataTransfer?.files;
		if (files && files.length > 0) {
			selectedFiles = Array.from(files);
			fileInput.files = files;
			fileInput.dispatchEvent(new Event('change'));
		}
	}

	function handleInputChange(event: Event) {
		const files = (event.target as HTMLInputElement).files;
		if (files && files.length > 0) {
			selectedFiles = Array.from(files);
		}
	}

	function removeFile(index: number) {
		selectedFiles = selectedFiles.filter((_, i) => i !== index);
		// Update file input (optional, for further processing)
		const dataTransfer = new DataTransfer();
		selectedFiles.forEach((file) => dataTransfer.items.add(file));
		fileInput.files = dataTransfer.files;
	}

	function onclick() {
		enqueueUpload(selectedFiles);
		open = false;
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="w-5/12">
		<Dialog.Header>
			<Dialog.Title>Upload Files</Dialog.Title>
		</Dialog.Header>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="outline-border relative h-72 w-full select-none rounded-lg outline-dashed outline-2 {isDragging
				? 'bg-blue-100 outline-blue-400'
				: ''} flex flex-col p-4"
			ondragover={handleDragOver}
			ondragleave={handleDragLeave}
			ondrop={handleDrop}
		>
			{#if selectedFiles.length > 0}
				<div class="relative z-10 max-h-full min-h-0 flex-1 space-y-2 overflow-y-auto pr-2">
					{#each selectedFiles as file, i}
						<div class="hover:bg-secondary flex items-center justify-between rounded px-2 py-1">
							<span class="truncate">{file.name}</span>
							<button
								type="button"
								class="ml-2 p-1 hover:text-red-500"
								onclick={() => removeFile(i)}
								aria-label="Remove file"
							>
								<X size={18} />
							</button>
						</div>
					{/each}
				</div>
			{/if}
			{#if selectedFiles.length === 0}
				<h1 class="absolute inset-0 flex items-center justify-center">Upload Files</h1>
			{/if}
			<input
				type="file"
				class="absolute inset-0 z-0 h-full w-full cursor-pointer opacity-0"
				multiple={true}
				bind:this={fileInput}
				onchange={handleInputChange}
			/>
		</div>

		<Button {onclick} disabled={selectedFiles.length === 0}
			>Upload {selectedFiles.length} files</Button
		>
	</Dialog.Content>
</Dialog.Root>
