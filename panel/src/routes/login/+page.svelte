<script lang="ts">
	import { goto } from '$app/navigation';
	import { signInUser } from '$lib/api';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card/index';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';

	let username = $state('');
	let password = $state('');
	let buttonEnabled = $derived(username.length > 3 && password.length > 5);
	let signInError = $state(false);

	$effect(() => {
		username || password;
		signInError = false;
	});

	async function onSignInClick() {
		try {
			await signInUser({ username, password });
			goto('/dash');
		} catch (err) {
			signInError = true;
		}
	}
</script>

<div class="flex h-screen w-full flex-col items-center justify-center">
	<Card.Root class="mx-4 w-[calc(100%-2rem)] md:w-3/6 lg:w-4/12 xl:w-3/12">
		<Card.Header>
			<Card.Title>Sign in to the admin panel</Card.Title>
		</Card.Header>

		<Card.Content>
			<div class="grid gap-1.5">
				<Label>Username</Label>
				<Input bind:value={username} type="email"></Input>
			</div>

			<div class="mt-2 grid gap-1.5">
				<Label>Password</Label>
				<Input bind:value={password} type="password"></Input>
			</div>

			{#if signInError}
				<p class="mt-2 text-red-500">Incorrect username or password</p>
			{/if}

			<Button onclick={onSignInClick} disabled={!buttonEnabled} class="mt-2 w-full">Sign in</Button>
		</Card.Content>
	</Card.Root>
</div>
