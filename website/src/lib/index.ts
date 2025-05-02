import { writable } from 'svelte/store';
import type { APIFile } from './api/files';

// place files you want to import through the `$lib` alias in this folder.
// export type FullAPIFile = APIFile & { _width: number; _height: number, _index: number };

export interface ImageRow {
	index: number;
	height: number;
	files: Array<ImageFile>;
}

export interface ImageFile {
	id: bigint;
	width: number;
	height: number;
}

export const scrollIntoBucket = writable<string>();
export const globalScrollTop = writable<number>();
