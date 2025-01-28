async function deriveKey(password: string, salt: ArrayBuffer): Promise<CryptoKey> {
	const encoder = new TextEncoder();
	const passwordKey = await window.crypto.subtle.importKey(
		'raw',
		encoder.encode(password),
		{ name: 'PBKDF2' },
		false,
		['deriveKey']
	);
	return window.crypto.subtle.deriveKey(
		{
			name: 'PBKDF2',
			salt,
			iterations: 600000,
			hash: 'SHA-256'
		},
		passwordKey,
		{ name: 'AES-GCM', length: 256 },
		false,
		['encrypt', 'decrypt']
	);
}

function indexToBytes(index: number): Uint8Array {
	const buffer = new ArrayBuffer(4);
	new DataView(buffer).setUint32(0, index, false); // Big-endian
	return new Uint8Array(buffer);
}

async function readChunk(file: Blob, start: number, end: number): Promise<ArrayBuffer> {
	return new Promise((resolve, reject) => {
		const reader = new FileReader();
		reader.onload = () => resolve(reader.result as ArrayBuffer);
		reader.onerror = reject;
		reader.readAsArrayBuffer(file.slice(start, end));
	});
}

export async function encryptFile(
	file: Blob,
	password: string,
	chunkSize = 1024 * 1024
): Promise<{
	encryptedData: Blob;
	salt: Uint8Array<ArrayBuffer>;
	fixedIv: Uint8Array;
	chunkSize: number;
}> {
	const salt = window.crypto.getRandomValues(new Uint8Array(16));
	const fixedIv = window.crypto.getRandomValues(new Uint8Array(8));
	const key = await deriveKey(password, salt.buffer);

	const totalChunks = Math.ceil(file.size / chunkSize);
	const encryptedChunks: ArrayBuffer[] = [];

	for (let i = 0; i < totalChunks; i++) {
		const start = i * chunkSize;
		const end = Math.min(file.size, start + chunkSize);
		const chunk = await readChunk(file, start, end);

		const iv = new Uint8Array([...fixedIv, ...indexToBytes(i)]);
		const encrypted = await window.crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, chunk);
		encryptedChunks.push(encrypted);
	}

	return {
		encryptedData: new Blob(encryptedChunks),
		salt,
		fixedIv,
		chunkSize
	};
}

export async function decryptFile(
	encryptedBlob: Blob,
	salt: Uint8Array<ArrayBuffer>,
	fixedIv: Uint8Array<ArrayBuffer>,
	chunkSize: number,
	password: string
): Promise<Blob> {
	const key = await deriveKey(password, salt.buffer);
	const encryptedSize = encryptedBlob.size;
	const tagLength = 16; // AES-GCM tag is 16 bytes
	const encryptedChunkSize = chunkSize + tagLength;
	const totalChunks = Math.ceil(encryptedSize / encryptedChunkSize);

	const decryptedChunks: ArrayBuffer[] = [];

	for (let i = 0; i < totalChunks; i++) {
		const start = i * encryptedChunkSize;
		const end = Math.min(encryptedSize, start + encryptedChunkSize);
		const encryptedChunk = await readChunk(encryptedBlob, start, end);

		const iv = new Uint8Array([...fixedIv, ...indexToBytes(i)]);
		const decrypted = await window.crypto.subtle.decrypt(
			{ name: 'AES-GCM', iv },
			key,
			encryptedChunk
		);
		decryptedChunks.push(decrypted);
	}

	return new Blob(decryptedChunks);
}
