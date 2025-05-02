import { PUBLIC_SERVER_URL } from '$env/static/public';
import { kyc } from '.';

export interface APIFile {
	id: bigint;
	mimeType: string;
	fileType: 'image' | 'video' | 'audio';
	fileName: string;
	albumId?: bigint;
	size: number;
	s3Key: string;
	width?: number;
	height?: number;
	duration?: number;
	exifData?: Record<string, any>;
	waveform?: number[];
	updatedAt: Date;
	createdAt: Date;
}

export interface TimeBucket {
	date: Date;
	count: number;
}

export async function uploadFile(file: File, albumId?: bigint) {
	const formData = new FormData();
	formData.append('file', file);
	formData.append('data', JSON.stringify({ albumId }));

	const response = await fetch(`${PUBLIC_SERVER_URL}/api/files/upload`, {
		method: 'POST',
		body: formData,
		credentials: 'include'
	});

	if (!response.ok) {
		throw new Error('Failed to upload file');
	}

	return await response.json();
}

export async function getBuckets(type: 'image' | 'video' | 'audio'): Promise<TimeBucket[]> {
	const response: TimeBucket[] = await kyc.get(`files/buckets?type=${type}`).json();
	return response.map((x) => ({ count: x.count, date: new Date(x.date) }));
}

export async function getBucketFiles(type: 'image' | 'video' | 'audio', date: string | Date) {
	if (date instanceof Date) {
		date = date.toISOString();
	}

	const response: APIFile[] = await kyc.get(`files/buckets/files?type=${type}&date=${date}`).json();
	return response;
}
