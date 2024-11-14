<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import PageContainer from './PageContainer.svelte';
	import { verify } from '$lib/api';
	import { goto } from '$app/navigation';

	interface Props {
		children: Snippet;
	}

	const { children }: Props = $props();

	onMount(async () => {
		if (!(await verify())) {
			goto('/login');
		}
	});
</script>

<PageContainer>
	{@render children()}
</PageContainer>
