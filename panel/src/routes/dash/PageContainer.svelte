<script lang="ts">
	import type { Snippet } from 'svelte';
	import PageSideBar from './PageSideBar.svelte';

	interface Props {
		children: Snippet;
	}

	const { children }: Props = $props();
</script>

<div id="pg_container" class="bg-zinc-700">
	<div id="pg_navbar" class="bg-background"></div>

	<div id="pg_sidebar" class="bg-background py-4">
		<PageSideBar></PageSideBar>
	</div>

	<div id="pg_content" class="bg-background p-6">
		{@render children()}
	</div>
</div>

<style>
	#pg_container {
		display: grid;
		grid-template-areas:
			'navbar navbar'
			'sidebar content';
		grid-template-rows: 4rem 1fr;
		grid-template-columns: 16rem 1fr;
		height: 100dvh;
		width: 100vw;
		gap: 1px;
	}

	#pg_navbar {
		grid-area: navbar;
		/* background-color: red; */
		height: 4rem;
		width: 100%;
	}

	#pg_sidebar {
		grid-area: sidebar;
		/* background-color: cyan; */
		height: 100%;
		width: 16rem;
	}

	#pg_content {
		grid-area: content;
		width: 100%;
		height: 100%;
		/* background-color: yellow; */
	}
</style>
