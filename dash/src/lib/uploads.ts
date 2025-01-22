import { writable, get } from 'svelte/store';
import { v4 } from 'uuid';

export interface ActiveUpload {
	files: File[];
	id: string;
	vaultId: number;
	status: 'pending' | 'uploading' | 'cancelled' | 'completed';
	abortControler: AbortController;
	currentFileName?: string;
	progress: number;
	totalProgress: number;
	totalFiles: number;
	completedFiles: number;
}

export const upStore = writable<ActiveUpload[]>([]);

class UploadManager {
	async uploadFile(
		file: File,
		vaultId: number,
		signal: AbortSignal,
		uploadId: string,
		fileIndex: number
	): Promise<void> {
		const formData = new FormData();
		formData.append('file', file);

		try {
			const xhr = new XMLHttpRequest();
			await new Promise<void>((resolve, reject) => {
				signal.addEventListener('abort', () => {
					xhr.abort();
					reject(new Error('Upload cancelled'));
				});

				xhr.upload.addEventListener('progress', (event) => {
					if (event.lengthComputable) {
						const fileProgress = (event.loaded / event.total) * 100;
						upStore.update((uploads) => {
							const idx = uploads.findIndex((u) => u.id === uploadId);
							if (idx !== -1) {
								const updatedUploads = [...uploads];
								const upload = updatedUploads[idx];
								upload.progress = fileProgress;

								const totalProgress =
									(upload.completedFiles * 100 + fileProgress) / upload.totalFiles;
								upload.totalProgress = totalProgress;

								return updatedUploads;
							}
							return uploads;
						});
					}
				});

				xhr.addEventListener('load', () => {
					if (xhr.status >= 200 && xhr.status < 300) {
						upStore.update((uploads) => {
							const idx = uploads.findIndex((u) => u.id === uploadId);
							if (idx !== -1) {
								const updatedUploads = [...uploads];
								updatedUploads[idx].completedFiles++;
								return updatedUploads;
							}
							return uploads;
						});
						resolve();
					} else {
						reject(new Error(`HTTP Error: ${xhr.status}`));
					}
				});

				xhr.addEventListener('error', () => {
					reject(new Error('Network error'));
				});

				xhr.open('POST', `http://localhost:8080/api/vaults/${vaultId}/upload`);
				xhr.setRequestHeader('Authorization', localStorage.getItem('token')!);
				xhr.send(formData);
			});
		} catch (error) {
			if (signal.aborted) {
				throw new Error('Upload cancelled');
			}
			throw error;
		}
	}

	async processUpload(uploadId: string) {
		upStore.update((uploads) => {
			return uploads.map((upload) =>
				upload.id === uploadId ? { ...upload, status: 'uploading' } : upload
			);
		});

		const upload = get(upStore).find((u) => u.id === uploadId);
		if (!upload) return;

		console.log(`upload: ${upload.id} starting upload...`);

		let queue: Promise<void>[] = [];

		for (let i = 0; i < upload.files.length; i++) {
			if (queue.length === 15) {
				await Promise.all(queue);
                queue = []
			}

			const file = upload.files[i];
			const currentUpload = get(upStore).find((u) => u.id === uploadId);
			if (!currentUpload) break;

			if (currentUpload.status === 'cancelled') {
				currentUpload.abortControler.abort();
				break;
			}

			upStore.update((uploads) =>
				uploads.map((upload) =>
					upload.id === uploadId ? { ...upload, currentFileName: file.name, progress: 0 } : upload
				)
			);

			queue.push(
				this.uploadFile(
					file,
					currentUpload.vaultId,
					currentUpload.abortControler.signal,
					currentUpload.id,
					i
				)
			);
		}

        if (queue.length > 0) {
            await Promise.all(queue);
            queue = []
        }

		const currentUpload = get(upStore).find((u) => u.id === uploadId);
		if (currentUpload && currentUpload.status !== 'cancelled') {
			upStore.update((uploads) =>
				uploads.map((upload) =>
					upload.id === uploadId ? { ...upload, status: 'completed' } : upload
				)
			);
		}

		setTimeout(() => {
			upStore.update((uploads) => uploads.filter((x) => x.id !== uploadId));
		}, 2000);
	}

	enqueueUpload(up: { files: File[]; vaultId: number }) {
		const id = v4();
		const newUpload: ActiveUpload = {
			...up,
			id,
			status: 'pending',
			abortControler: new AbortController(),
			progress: 0,
			totalProgress: 0,
			totalFiles: up.files.length,
			completedFiles: 0
		};

		upStore.update((uploads) => [...uploads, newUpload]);
		this.processUpload(id);
		return id;
	}

	cancelUpload(id: string) {
		upStore.update((uploads) =>
			uploads.map((upload) => (upload.id === id ? { ...upload, status: 'cancelled' } : upload))
		);
	}
}

export const uploader = new UploadManager();
