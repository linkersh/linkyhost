<script lang="ts">
	import Label from '$lib/components/ui/label/label.svelte';

	const routes = [
		{
			name: 'Images',
			href: '/dash/images'
		},
		{
			name: 'Uploader',
			href: '/dash/uploader'
		}
	];
</script>

<div class="flex flex-col px-6">
	{#each routes as route}
		<a href={route.href} class="p-2 hover:cursor-pointer">
			<!-- side note: why the fuck is hover not working if parent is hovered -->
			<Label class="text-base text-secondary-foreground hover:cursor-pointer">{route.name}</Label>
		</a>
	{/each}
</div>
