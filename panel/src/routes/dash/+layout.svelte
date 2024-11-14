<script lang="ts">
	import type { Snippet } from 'svelte';
	import PageContainer from './PageContainer.svelte';

	interface Props {
		children: Snippet;
	}

	const { children }: Props = $props();
</script>

<PageContainer>
	{@render children()}
</PageContainer>
