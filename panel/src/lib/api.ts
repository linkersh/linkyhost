import { dev } from '$app/environment';

export const HTTP_API_URL = dev ? 'http://127.0.0.1:9810' : window.location.hostname;

export async function signInUser({ username, password }: { username: string; password: string }) {
	const request = await fetch(`${HTTP_API_URL}/api/user/signin`, {
		method: 'POST',
		body: JSON.stringify({ username, password }),
		headers: { 'Content-Type': 'application/json' },
		credentials: 'include'
	});

	if (request.status === 200) {
		return true;
	}
	return false;
}

export async function verify(): Promise<boolean> {
	const request = await fetch(`${HTTP_API_URL}/api/user/verify`, {
		method: 'GET',
		headers: { 'Content-Type': 'application/json' },
		credentials: 'include'
	});

	return request.status === 200;
}
