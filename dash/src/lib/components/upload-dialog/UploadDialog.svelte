<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { uploader } from '@/uploads';
	import Button from '../ui/button/button.svelte';
	import FileList from './FileList.svelte';
	import { Plus } from 'lucide-svelte';
	import { beginUpload } from '@/api/vaults';

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

	async function uploadChunk(uploadId: string, chunk: Blob) {
		const formData = new FormData();
		formData.append('data', chunk);

		const res = await fetch(`http://127.0.0.1:8080/api/vaults/uploads/${uploadId}/chunk`, {
			method: 'POST',
			headers: {
				Authorization: localStorage.getItem('token') || '',
				// 'Content-Type': 'multipart/form-data'
			},
			body: formData
		});

		if (!res.ok) {
			throw new Error('Failed to upload chunk');
		}
	}

	async function uploadFile(file: File) {
		const { id: uploadId } = await beginUpload({
			vaultId,
			contentType: file.type,
			fileName: file.name,
			fileSize: file.size
		});

		const CHUNK_SIZE = 90 * 1024 * 1024; // 90MB
		const totalChunks = Math.ceil(file.size / CHUNK_SIZE);

		for (let chunkIndex = 0; chunkIndex < totalChunks; chunkIndex++) {
			const start = chunkIndex * CHUNK_SIZE;
			const end = Math.min(start + CHUNK_SIZE, file.size);
			const chunk = file.slice(start, end);
			await uploadChunk(uploadId, chunk);
		}
	}

	async function startUpload() {
		for (const file of files) {
			await uploadFile(file);
		}

		// uploader.enqueueUpload({ files, vaultId });
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
