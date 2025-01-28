<script lang="ts">
	import type { VaultFile } from '@/api/vaults';
	import { File as FileIcon, AppWindow, HardDrive, Archive } from 'lucide-svelte';

	const file: VaultFile = $props();
</script>

<div
	class="bg-secondary relative m-2 flex min-h-56 w-full flex-col items-center gap-10 rounded-lg p-6 ring-1 ring-zinc-700"
>
	{#if file.content_type === 'application/x-msdownload'}
		<AppWindow size={144}></AppWindow>
	{:else if file.content_type === 'application/x-iso9660-image'}
		<HardDrive size={144}></HardDrive>
	{:else if ['application/x-gzip', 'application/x-bzip', 'application/x-bzip2', 'application/x-zip-compressed', 'application/zip', 'application/zip-compressed'].includes(file.content_type)}
		<Archive size={144}></Archive>
	{:else}
		<FileIcon size={144}></FileIcon>
	{/if}

	<span class="text-foreground absolute bottom-4 max-w-56 truncate break-words text-sm">
		{file.file_name}
	</span>
</div>
