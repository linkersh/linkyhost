import { kyc } from './client';

export interface LoginInfo {
	username: string;
	password: string;
}

export type LoginResult =
	| {
			status: 'error';
	  }
	| { status: 'success'; token: string };

export interface CreateUserInfo {
	username: string;
	password: string;
}

export const userLogin = async (info: LoginInfo): Promise<LoginResult> => {
	try {
		const response = await kyc.post('users/login', { json: info });
		const { token }: { token: string } = await response.json();
		return { status: 'success', token };
	} catch (err) {
		return { status: 'error' };
	}
};

export const userCreate = async (info: LoginInfo): Promise<LoginResult> => {
	const response = await kyc.post('users/login', { json: info });
	if (response.status !== 200) {
		return { status: 'error' };
	}

	const { token }: { token: string } = await response.json();
	return { status: 'success', token };
};
