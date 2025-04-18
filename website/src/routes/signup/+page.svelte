<script lang="ts">
	import { oauthRedirect } from '$lib/api/auth';
	import { Button } from '$lib/components/ui/button';
	import { Github } from '@lucide/svelte';
</script>

<div
	class="fixed left-1/2 top-1/2 flex -translate-x-1/2 -translate-y-1/2 flex-col items-center gap-2"
>
	<h1 class="text-xl font-semibold">Welcome, continue with</h1>
	<Button onclick={() => oauthRedirect('github')} variant="outline" class="w-64">
		<Github class="mr-2 h-4 w-4" />
		Sign in with GitHub
	</Button>
</div>
