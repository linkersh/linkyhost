import { writable } from 'svelte/store';
import { type VaultFile, type Vault, fetchFiles } from './api/vaults';

export const vaultStore = writable<Vault[]>([]);
export const activeVault = writable<Vault | undefined>();
export const activeVaultFiles = writable<VaultFile[]>([]);

export async function refreshActiveVaultFiles({
	vaultId,
	limit,
	skip
}: {
	vaultId: number;
	limit: number;
	skip: number;
}): Promise<VaultFile[]> {
	const files = await fetchFiles({ limit, skip, vaultId });
	activeVaultFiles.set(files);
	return files;
}
