import { kyc } from './client';

export interface Vault {
	id: number;
	name: string;
	user_id: number;
	created_at: string;
	is_encrypted: boolean;
}

export interface VaultFile {
	id: number;
	vault_id: number;
	user_id: number;
	file_name: string;
	size: number;
	created_at: string;
	uploaded_at: string;
	content_type: string;
	is_encrypted: boolean;
	is_hidden: boolean;
	fixed_iv: number[];
	password_salt: number[];
	chunk_size: number;
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
	is_encrypted: boolean;
}

export async function createVault({ name, is_encrypted }: CreateVaultInfo): Promise<Vault> {
	const vault = await kyc.post<Vault>(`vaults/create`, { json: { name, is_encrypted } }).json();
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

export interface DownloadFileInfo {
	vaultId: number;
	fileId: number;
}

export async function downloadFile({ vaultId, fileId }: DownloadFileInfo): Promise<Blob> {
	const response = await kyc.get(`vaults/${vaultId}/files/${fileId}/download`);
	return await response.blob();
}
