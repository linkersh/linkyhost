<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import AppSidebar from '$lib/components/AppSidebar.svelte';
	import { onDestroy, onMount } from 'svelte';
	import { uploader, upStore } from '@/uploads';
	import Upload from './Upload.svelte';

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
			{#each $upStore as upload}
				<Upload {...upload}></Upload>
			{/each}
		</div>
	</main>
</Sidebar.Provider>
