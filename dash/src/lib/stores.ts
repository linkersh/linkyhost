import { writable } from 'svelte/store';
import { type VaultFile, type Vault, fetchFiles } from './api/vaults';

export const vaultStore = writable<Vault[]>([]);
export const activeVault = writable<Vault | undefined>();
