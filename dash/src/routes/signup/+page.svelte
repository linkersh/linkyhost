<script lang="ts">
	import { Input } from '@/components/ui/input';
	import { userCreate } from '@/api';
	import { goto } from '$app/navigation';
	import * as Card from '$lib/components/ui/card/';
	import Button from '@/components/ui/button/button.svelte';
	import Label from '@/components/ui/label/label.svelte';

	let username = $state('');
	let password = $state('');
	let error = $state('');
	let isLoggingIn = $state(false);
	let isEnabled = $derived(username.length > 2 && password.length > 8 && !isLoggingIn);

	$effect(() => {
		username || password;
		error = '';
	});

	async function login() {
		isLoggingIn = true;
		try {
			const response = await userCreate({ username, password });
			if (response.status === 'error') {
				error = 'Invalid username or password';
			} else {
				localStorage.setItem('token', response.token);
				goto('/dash');
			}
		} catch (err) {
			console.error(err);
		}
		isLoggingIn = false;
	}
</script>

<div class="flex h-screen w-screen flex-col items-center justify-center">
	<Card.Root class="w-full md:w-3/4 lg:w-2/6">
		<Card.Header>
			<Card.Title>Sign up to linykhost</Card.Title>
		</Card.Header>

		<Card.Content>
			<div class="grid gap-1.5">
				<Label>Username</Label>
				<Input type="email" bind:value={username} />
			</div>

			<div class="mt-4 grid gap-1.5">
				<Label>Password</Label>
				<Input type="password" bind:value={password} />
			</div>

			{#if error}
				<p class="mt-4 text-red-500">{error}</p>
			{/if}

			<Button disabled={!isEnabled} onclick={login} class="mt-4 w-full">Create Account</Button>

			<p class="text-muted-foreground mt-4 text-sm">
				Already have an account? <a class="underline" href="/login">Login</a> instead.
			</p>
		</Card.Content>
	</Card.Root>
</div>
