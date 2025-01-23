<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import AppSidebar from '$lib/components/AppSidebar.svelte';
	import Upload from './Upload.svelte';
	import { onDestroy, onMount } from 'svelte';
	import { uploadStore } from '@/uploads';
	// import { uploadStore, type ActiveUpload } from '@/uploads';

	let { children } = $props();
</script>

<Sidebar.Provider>
	<AppSidebar />
	<main class="w-full">
		<Sidebar.Trigger />
		<div class="p-8">
			{@render children?.()}
		</div>

		<div class="fixed bottom-4 right-4 flex flex-col gap-2">
			{#each $uploadStore as upload}
				<Upload {...upload}></Upload>
			{/each}
		</div>
	</main>
</Sidebar.Provider>
