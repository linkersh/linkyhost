import { kyc } from './client';

export interface Vault {
	id: number;
	name: string;
	user_id: number;
	kind: number;
	created_at: string;
	flags: number;
}

export interface VaultFile {
	id: number;
	vault_id: number;
	user_id: number;
	name: string;
	size: number;
	created_at: string;
	uploaded_at: string;
	content_type: string;
}

export async function fetchVaults(): Promise<Vault[]> {
	const vaults = await kyc.get<Vault[]>('vaults').json();
	return vaults;
}

export async function fetchFiles({
	vaultId,
	limit,
	skip
}: {
	vaultId: number;
	limit: number;
	skip: number;
}): Promise<VaultFile[]> {
	const files = await kyc
		.get<VaultFile[]>(`vaults/${vaultId}/files?limit=${limit}&skip=${skip}`)
		.json();
	return files;
}

export enum VaultFlags {
	Encrypted = 1,
	OCR = 2
}

export interface CreateVaultInfo {
	name: string;
	flags: number;
}

export async function createVault({ name, flags }: CreateVaultInfo): Promise<Vault> {
	const vault = await kyc.post<Vault>(`vaults/create`, { json: { name, flags, kind: 2 } }).json();
	return vault;
}

export interface DeleteVaultInfo {
	id: number;
}

export async function deleteVault({ id }: DeleteVaultInfo): Promise<void> {
	await kyc.post<Vault>(`vaults/${id}/delete`, { json: {} }).json();
}

export interface StartUploadResp {
	id: string;
}

export async function beginUpload({
	vaultId,
	fileName,
	fileSize,
	contentType
}: {
	vaultId: number;
	fileName: string;
	fileSize: number;
	contentType: string;
}): Promise<StartUploadResp> {
	return await kyc
		.post<StartUploadResp>(`vaults/${vaultId}/beginUpload`, {
			json: { file_name: fileName, file_size: fileSize, content_type: contentType }
		})
		.json();
}

// export interface GroupUploadFile {
// 	file: File;
// }

// export async function groupUpload({}) {}
