import { writable } from 'svelte/store';
import type { Vault } from './api/vaults';
import type { ActiveUpload } from './uploads';

export const vaultStore = writable<Vault[]>([]);
export const activeVault = writable<Vault | undefined>();
