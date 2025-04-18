import { uploadFile } from './api/files';

type UploadStatus = 'queued' | 'uploading' | 'completed' | 'error';

interface QueueFile {
	file: File;
	status: UploadStatus;
	error?: string;
}

interface QueueUpload {
	id: string;
	files: QueueFile[];
	status: UploadStatus;
}

const queue: QueueUpload[] = [];
let isProcessing = false;

function createUploadID() {
	return crypto.randomUUID();
}

export function getUploadStatus(id: string) {
	return queue.find((upload) => upload.id === id);
}

export async function enqueueUpload(files: File[]) {
	const uploadId = createUploadID();
	const upload: QueueUpload = {
		id: uploadId,
		files: files.map((file) => ({ file, status: 'queued' })),
		status: 'queued'
	};
	queue.push(upload);

	processQueue();
	return uploadId;
}

async function processQueue(): Promise<void> {
	if (isProcessing || queue.length === 0) return;
	isProcessing = true;

	try {
		const upload = queue[0];
		if (!upload) {
			isProcessing = false;
			return;
		}

		upload.status = 'uploading';

		for (const queueFile of upload.files) {
			try {
				queueFile.status = 'uploading';
				await uploadFile(queueFile.file);
				queueFile.status = 'completed';
			} catch (error) {
				queueFile.status = 'error';
				queueFile.error = error instanceof Error ? error.message : 'Upload failed';
			}
		}

		// upload.status = upload.files.some((f) => f.status === 'error') ? 'error' : 'completed';
		queue.shift();

		if (queue.length > 0) {
			await processQueue();
		}
	} finally {
		isProcessing = false;
	}
}
