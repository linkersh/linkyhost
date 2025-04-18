import { PUBLIC_SERVER_URL } from '$env/static/public';
import ky from 'ky';

export const kyc = ky.extend({
	prefixUrl: `${PUBLIC_SERVER_URL}/api/`,
	credentials: 'include'
});
