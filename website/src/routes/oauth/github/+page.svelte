<script lang="ts">
	import { PUBLIC_SERVER_URL } from '$env/static/public';
	import { onMount } from 'svelte';

	onMount(() => {
		window.location.href = `${PUBLIC_SERVER_URL}/api/auth/github/callback${window.location.search}`;
	});
</script>
