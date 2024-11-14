import { dev } from '$app/environment';

export const HTTP_API_URL = dev ? 'http://127.0.0.1:9810' : window.location.hostname;

export async function signInUser({ username, password }: { username: string; password: string }) {
	const request = await fetch(`${HTTP_API_URL}/api/user/signin`, {
		method: 'POST',
		body: JSON.stringify({ username, password }),
		headers: { 'Content-Type': 'application/json' }
	});

	console.log(await request.json());
}
