import { getBucketFiles, getBuckets, type APIFile, type TimeBucket } from './api/files';

interface TimeBucketDate {
	date: string;
	count: number;
	dateMs: number;
}

export interface BucketWithFiles {
	bucketId: string;
	files: APIFile[];
}

class Gallery {
	buckets: TimeBucketDate[];
	loadedFiles: Map<string, APIFile[]> = new Map();
	loaddedBuckets: TimeBucketDate[] = [];

	focuson: (buckets: TimeBucket) => void = () => {};

	constructor(buckets: TimeBucket[]) {
		this.buckets = buckets.map((x) => ({
			date: x.date,
			count: x.count,
			dateMs: new Date(x.date).getTime()
		}));
		this.setCurrentBucket(buckets[0]);
	}

	private hasBucket(date: string) {
		return this.buckets.some((bucket) => bucket.date === date);
	}

	private isBucketLoaded(date: string) {
		return this.loaddedBuckets.some((bucket) => bucket.date === date);
	}

	private addLoadedBucket(bucket: TimeBucket) {
		if (this.hasBucket(bucket.date) && !this.isBucketLoaded(bucket.date)) {
			this.loaddedBuckets.push({
				date: bucket.date,
				count: bucket.count,
				dateMs: new Date(bucket.date).getTime()
			});
			this.loaddedBuckets.sort((a, b) => b.dateMs - a.dateMs);
			console.log('LOADED BUCKETS:', this.loaddedBuckets);
		}
	}

	public async getLoadedBuckets(): Promise<BucketWithFiles[]> {
		const buckets: BucketWithFiles[] = [];
		for (const bucket of this.loaddedBuckets) {
			if (this.loadedFiles.has(bucket.date)) {
				const files = this.loadedFiles.get(bucket.date)!;
				buckets.push({ bucketId: bucket.date, files });
			} else {
				const files = await getBucketFiles('image', bucket.date);
				this.loadedFiles.set(bucket.date, files);
				buckets.push({ bucketId: bucket.date, files });
			}
		}
		return buckets;
	}

	public getNewer() {
		const newest = this.loaddedBuckets[0];
		const beforeThat = this.buckets.reverse().find((x) => x.dateMs < newest.dateMs);
		if (beforeThat) {
			this.setCurrentBucket(beforeThat);
		}
	}

	public getOlder() {
		const oldest = this.loaddedBuckets[this.loaddedBuckets.length - 1];
		const afterThat = this.buckets.find((x) => x.dateMs > oldest.dateMs);
		if (afterThat) {
			this.setCurrentBucket(afterThat);
		}
	}

	public setCurrentBucket(bucket: TimeBucket) {
		this.addLoadedBucket(bucket);
		this.focuson(bucket);
	}
}

let gallery: Gallery;
export async function getGallery() {
	if (gallery) return gallery;

	gallery = new Gallery(await getBuckets('image'));
	return gallery;
}
