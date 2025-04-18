<script lang="ts">
	import { ModeWatcher } from 'mode-watcher';
	import '../app.pcss';

	let { children } = $props();
</script>

<ModeWatcher></ModeWatcher>

<div class=" h-screen w-screen p-4">
	{@render children()}
</div>
