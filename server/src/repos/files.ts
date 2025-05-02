import { db, files } from "@/db";
import type { PgTrans } from "@/utils";
import { and, count, desc, eq, gte, inArray, lte, sql } from "drizzle-orm";
import { DateTime } from "luxon";

interface CreateFileBaseOptions {
  id: bigint;
  type: "image" | "video" | "audio";
  file: {
    name: string;
    contentType: string;
  };
  userId: bigint;
  albumId?: bigint | null;
  size: number;
  s3Key: string;
  createdAt: Date;
}

export interface TimeBucket {
  date: string;
  count: number;
}

export interface GetTimeBucketsOptions {
  type: "image" | "video" | "audio";
  userId: bigint;
}

export interface GetBucketFilesOptions {
  type: "image" | "video" | "audio";
  date: string;
  userId: bigint;
}

export type CreateImageOptions = CreateFileBaseOptions & {
  width: number;
  height: number;
  duration?: number;
  exifData?: Record<string, any>;
};

export type CreateAudioOptions = CreateFileBaseOptions & {
  duration: number;
  waveform: number[];
  exifData?: Record<string, any>;
};

export class FileRepository {
  tx?: PgTrans;

  constructor(tx?: PgTrans) {
    this.tx = tx;
  }

  get exec() {
    return this.tx ?? db;
  }

  public async createFile(options: CreateImageOptions | CreateAudioOptions) {
    const { file, userId, albumId, size, s3Key, createdAt, id, type } = options;
    const [newFile] = await this.exec
      .insert(files)
      .values({
        id,
        userId,
        albumId,
        fileName: file.name,
        mimeType: file.contentType,
        fileType: type,
        size,
        s3Key,
        createdAt,
        width: "width" in options ? options.width : null,
        height: "height" in options ? options.height : null,
        duration: "duration" in options ? options.duration : null,
        exifData: "exifData" in options ? options.exifData : null,
        waveform: "waveform" in options ? options.waveform : null,
      })
      .returning();

    return newFile!;
  }

  public async getTimeBuckets({ userId, type }: GetTimeBucketsOptions) {
    const truncatedDate = sql<string>`date_trunc('month', ${files.createdAt})::text`;
    const buckets = await this.exec
      .select({
        date: truncatedDate,
        count: count(),
      })
      .from(files)
      .where(and(eq(files.userId, userId), eq(files.fileType, type)))
      .groupBy(truncatedDate)
      .orderBy(desc(truncatedDate));

    return buckets;
  }

  public async getBucketFiles({ userId, type, date }: GetBucketFilesOptions) {
    // Parse the ISO date string to get the start and end of the month
    const dateObj = new Date(date);
    const startOfMonth = new Date(dateObj.getFullYear(), dateObj.getMonth(), 1);
    const endOfMonth = new Date(
      dateObj.getFullYear(),
      dateObj.getMonth() + 1,
      0,
      23,
      59,
      59,
      999
    );

    const bucketFiles = await this.exec
      .select()
      .from(files)
      .where(
        and(
          eq(files.userId, userId),
          eq(files.fileType, type),
          gte(files.createdAt, startOfMonth),
          lte(files.createdAt, endOfMonth)
        )
      )
      .orderBy(desc(files.createdAt));

    return bucketFiles;
  }

  public async getFile({ id, userId }: { id: bigint; userId: bigint }) {
    const file = await this.exec.query.files.findFirst({
      where: and(eq(files.userId, userId), eq(files.id, id)),
    });
    return file;
  }

  public async getFiles(userId: bigint) {
    const returnedFiles = await this.exec
      .select()
      .from(files)
      .where(eq(files.userId, userId))
      .orderBy(desc(files.createdAt));
    return returnedFiles;
  }
}
