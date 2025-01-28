<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Plus } from 'lucide-svelte';
	import { uploader } from '@/uploads';
	import Button from '../ui/button/button.svelte';
	import FileList from './FileList.svelte';

	let isDragging = $state(false);
	let dialogOpen = $state(false);
	let files = $state<File[]>([]);
	let { vaultId }: { vaultId: number } = $props();

	type DragOverEv = DragEvent & { currentTarget: EventTarget & HTMLButtonElement };
	type DragLeaveEv = DragOverEv;
	type DropEv = DragEvent & { currentTarget: EventTarget & HTMLButtonElement };
	type InputEv = Event & { currentTarget: EventTarget & HTMLButtonElement };

	function onDragOver(ev: DragOverEv) {
		ev.preventDefault();
		ev.stopPropagation();
		isDragging = true;
	}

	function onDragLeave(ev: DragLeaveEv) {
		ev.preventDefault();
		ev.stopPropagation();
		isDragging = false;
	}

	function onDrop(ev: DropEv) {
		ev.preventDefault();
		ev.stopPropagation();
		isDragging = false;

		if (ev.dataTransfer && ev.dataTransfer.files) {
			addFiles(Array.from(ev.dataTransfer.files));
		}
	}

	function onInput(ev: InputEv) {
		const target = ev.target as HTMLInputElement;
		if (target.files) {
			addFiles(Array.from(target.files));
		}
	}

	function addFiles(newFiles: File[]) {
		files = [...files, ...newFiles];
	}

	async function startUpload() {
		uploader.addUpload(vaultId, files, {
			enabled: true,
			password: sessionStorage.getItem(`vault_${vaultId}_password`)!
		});
		dialogOpen = false;
		files = [];
	}
</script>

<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Trigger>
		<Button
			onclick={() => {
				dialogOpen = true;
			}}
		>
			<Plus></Plus>
			Upload
		</Button>
	</Dialog.Trigger>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Upload your files</Dialog.Title>
		</Dialog.Header>

		<button
			class="outline-secondary relative flex h-56 w-full flex-row items-center justify-center rounded-lg outline-dashed {isDragging
				? 'outline-primary'
				: ''} transition-colors"
			ondragover={onDragOver}
			ondragleave={onDragLeave}
			ondrop={onDrop}
			oninput={onInput}
		>
			<h1>Drag & drop or click inside this field</h1>

			<input class="absolute left-0 top-0 h-full w-full opacity-0" type="file" multiple={true} />
		</button>

		<FileList bind:files></FileList>
		<Button onclick={startUpload} disabled={files.length === 0}>Upload {files.length} files</Button>
	</Dialog.Content>
</Dialog.Root>
