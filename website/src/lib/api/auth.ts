import { PUBLIC_SERVER_URL } from '$env/static/public';

export function oauthRedirect(type: 'github') {
	const url = `${PUBLIC_SERVER_URL}/api/auth/${type}`;
	window.location.href = url;
}
