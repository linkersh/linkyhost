export class CredManager {
	vaultPwdMap: Map<number, string> = new Map();

	public addPassword(vaultId: number, pass: string) {
		this.vaultPwdMap.set(vaultId, pass);
	}

	public getPassword(vaultId: number): string | undefined {
		return this.vaultPwdMap.get(vaultId);
	}

    public hasPassword(vaultId: number): boolean {
        return this.vaultPwdMap.has(vaultId);
    }
}

export const credManager = new CredManager();
