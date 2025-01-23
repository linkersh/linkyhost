import { writable } from 'svelte/store';
import { v4 } from 'uuid';
import { beginUpload } from './api/vaults';

export const CHUNK_SIZE = 90 * 1024 * 1024;

export interface GroupUploadMeta {
	file_name: string;
	file_size: number;
	content_type: string;
}

export type ActiveUploadStatus = 'pending' | 'uploading' | 'cancelled' | 'completed' | 'errored';

export interface ActiveUpload {
	id: string;
	vaultId: number;
	files: File[];
	status: ActiveUploadStatus;
	abortController: AbortController;
	totalSize: number;
	transferredSize: number;
	currentFileName?: string;
}

export const uploadStore = writable<ActiveUpload[]>([]);

export class UploadManager {
	private queue: ActiveUpload[] = [];
	private isProcessing: boolean = false;

	async processQueue() {
		if (this.isProcessing || this.queue.length === 0) {
			return;
		}

		const upload = this.queue.shift()!;
		this.updateUploadState(upload.id, (up) => {
			up.status = 'uploading';
			return up;
		});

		try {
			let metadata: GroupUploadMeta[] = [];
			let tempFileStore: File[] = [];
			let totalFileSize = 0;

			for (const file of upload.files) {
				if (file.size < CHUNK_SIZE) {
					// lets collect more files :3
					tempFileStore.push(file);
					totalFileSize += file.size;

					const fileMetadata = {
						file_name: file.name,
						file_size: file.size,
						content_type: file.type
					};
					metadata.push(fileMetadata);

					const stringifiedJson = JSON.stringify(metadata);
					if (stringifiedJson.length + totalFileSize < CHUNK_SIZE) {
						continue;
					} else if (stringifiedJson.length + totalFileSize > CHUNK_SIZE) {
						tempFileStore.pop();
						metadata.pop();

						await this.uploadGroup(
							upload.vaultId,
							upload.id,
							upload.abortController.signal,
							metadata,
							tempFileStore
						);

						if (upload.abortController.signal.aborted) {
							break;
						}

						tempFileStore = [file];
						metadata = [fileMetadata];
						totalFileSize = file.size;
						continue;
					}
				}

				await this.uploadFile(file, upload.id, upload.abortController.signal, upload.vaultId);
				if (upload.abortController.signal.aborted) {
					break;
				}
			}

			if (tempFileStore.length > 0) {
				await this.uploadGroup(
					upload.vaultId,
					upload.id,
					upload.abortController.signal,
					metadata,
					tempFileStore
				);
			}

			const status = upload.abortController.signal.aborted ? 'cancelled' : 'completed';
			this.updateUploadState(upload.id, (up) => {
				up.status = status;
				return up;
			});
		} catch (err) {
			console.error(err);
			this.updateUploadState(upload.id, (up) => {
				up.status = 'errored';
				return up;
			});
		} finally {
			this.isProcessing = false;
			setTimeout(() => {
				uploadStore.update((up) => {
					const idx = up.findIndex((x) => x.id === upload.id);
					up.splice(idx, 1);
					return up;
				});
			}, 3000);
		}
	}

	async uploadFile(file: File, uploadId: string, abortSignal: AbortSignal, vaultId: number) {
		const { id: operationId } = await beginUpload({
			vaultId,
			contentType: file.type,
			fileName: file.name,
			fileSize: file.size
		});

		this.updateUploadState(uploadId, (up) => {
			up.currentFileName = file.name;
			return up;
		});

		try {
			const totalChunks = Math.ceil(file.size / CHUNK_SIZE);

			for (let chunkIndex = 0; chunkIndex < totalChunks; chunkIndex++) {
				const start = chunkIndex * CHUNK_SIZE;
				const end = Math.min(start + CHUNK_SIZE, file.size);
				const chunk = file.slice(start, end);
				await this.uploadChunk(uploadId, operationId, abortSignal, chunk);
				if (abortSignal.aborted) {
					break;
				}
			}
		} catch (err) {
			throw err;
		} finally {
			this.updateUploadState(uploadId, (up) => {
				up.currentFileName = undefined;
				return up;
			});
		}
	}

	async uploadGroup(
		vaultId: number,
		uploadId: string,
		abortSignal: AbortSignal,
		metadata: GroupUploadMeta[],
		files: File[]
	) {
		console.log('[GROUP UPLOAD] attempting to upload', files.length, 'files');

		const formData = new FormData();
		const url = `http://127.0.0.1:8080/api/vaults/${vaultId}/groupUpload`;

		for (const f of files) {
			formData.append('data', f);
		}
		formData.append('metadata', JSON.stringify(metadata));

		return new Promise((resolve, reject) => {
			const xhr = new XMLHttpRequest();
			let previousLoaded = 0;

			xhr.upload.addEventListener('progress', (event) => {
				if (event.lengthComputable) {
					const incrementalBytesTransferred = event.loaded - previousLoaded;
					previousLoaded = event.loaded;

					this.updateUploadState(uploadId, (up) => {
						up.transferredSize = up.transferredSize + incrementalBytesTransferred;
						return up;
					});
				}
			});

			xhr.addEventListener('load', () => {
				if (xhr.status >= 200 && xhr.status < 300) {
					resolve(xhr.response);
				} else {
					reject(new Error('Failed to upload chunk'));
				}
			});

			xhr.addEventListener('error', () => {
				reject(new Error('Failed to upload chunk'));
			});

			abortSignal.addEventListener('abort', () => {
				xhr.abort();
				resolve(undefined);
			});

			xhr.open('POST', url, true);
			xhr.setRequestHeader('Authorization', localStorage.getItem('token') || '');
			xhr.send(formData);
		});
	}

	async uploadChunk(uploadId: string, operationId: string, abortSignal: AbortSignal, chunk: Blob) {
		const formData = new FormData();
		const url = `http://127.0.0.1:8080/api/vaults/uploads/${operationId}/chunk`;
		formData.append('data', chunk);

		return new Promise((resolve, reject) => {
			const xhr = new XMLHttpRequest();
			let previousLoaded = 0;

			xhr.upload.addEventListener('progress', (event) => {
				if (event.lengthComputable) {
					const incrementalBytesTransferred = event.loaded - previousLoaded;
					previousLoaded = event.loaded;

					this.updateUploadState(uploadId, (up) => {
						up.transferredSize = up.transferredSize + incrementalBytesTransferred;
						return up;
					});
				}
			});

			xhr.addEventListener('load', () => {
				if (xhr.status >= 200 && xhr.status < 300) {
					resolve(xhr.response);
				} else {
					reject(new Error('Failed to upload chunk'));
				}
			});

			xhr.addEventListener('error', () => {
				reject(new Error('Failed to upload chunk'));
			});

			abortSignal.addEventListener('abort', () => {
				xhr.abort();
				resolve(undefined);
			});

			xhr.open('POST', url, true);
			xhr.setRequestHeader('Authorization', localStorage.getItem('token') || '');
			xhr.send(formData);
		});
	}

	updateUploadState(uploadId: string, transform: (up: ActiveUpload) => ActiveUpload) {
		uploadStore.update((u) => {
			const index = u.findIndex((x) => x.id === uploadId);
			if (index > -1) {
				const up = u[index];
				u[index] = transform(up);
			}
			return u;
		});
	}

	addUpload(files: File[], vaultId: number) {
		const id = v4();
		let totalSize = 0;
		for (const file of files) {
			totalSize += file.size;
		}

		const upload: ActiveUpload = {
			id,
			vaultId,
			files,
			totalSize,
			abortController: new AbortController(),
			status: 'pending',
			transferredSize: 0
		};

		uploadStore.update((u) => {
			u.push(upload);
			return u;
		});

		this.queue.push(upload);
		this.processQueue();
		return id;
	}

	cancelUpload(uploadId: string) {
		this.updateUploadState(uploadId, (up) => {
			up.status = 'cancelled';
			up.abortController.abort();
			return up;
		});
	}
}

export const uploader = new UploadManager();
