import { goto } from '$app/navigation';
import ky from 'ky';

export const kyc = ky.create({
	prefixUrl: 'http://localhost:8080/api',
	hooks: {
		beforeRequest: [
			async (request) => {
				const token = localStorage.getItem('token');
				if (token) {
					request.headers.set('Authorization', token);
				}
			}
		],
		afterResponse: [
			(_req, _opt, response) => {
				if (response.status === 403) {
					goto('/login');
				}
			}
		]
	}
});
