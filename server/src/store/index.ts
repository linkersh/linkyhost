import log from "@/utils/log";
import {
  GetObjectCommand,
  PutObjectCommand,
  DeleteObjectCommand,
  S3Client,
} from "@aws-sdk/client-s3";

const endpoint = Bun.env.S3_URL!;
const BUCKET = Bun.env.S3_BUCKET!;

export const s3Client = new S3Client({
  endpoint,
  region: Bun.env.S3_REGION!,
  credentials: {
    accessKeyId: Bun.env.S3_USER!,
    secretAccessKey: Bun.env.S3_PASSWORD!,
  },
  forcePathStyle: true,
});

export class CreatedFile {
  public key: string;
  private marked: boolean = false;

  constructor(key: string) {
    this.key = key;
  }

  async [Symbol.asyncDispose]() {
    if (this.marked) {
      return;
    }
    try {
      await deleteObject(this.key);
    } catch (err) {
      log.error(`Failed to delete file ${this.key}: ${err}`);
    }
  }

  mark() {
    this.marked = true;
  }
}

type FileType = "image" | "video" | "audio";

async function put(
  type: FileType,
  fileId: bigint,
  mimeType: string,
  file: Uint8Array
) {
  const key = `${type}s/${fileId}`;
  const command = new PutObjectCommand({
    Bucket: BUCKET,
    Key: key,
    Body: file,
    ContentType: mimeType,
  });
  await s3Client.send(command);
  return new CreatedFile(key);
}

async function get(type: FileType, fileId: bigint) {
  const key = `${type}s/${fileId}`;
  const command = new GetObjectCommand({
    Bucket: BUCKET,
    Key: key,
  });
  const response = await s3Client.send(command);
  if (!response.Body) {
    throw new Error("No body in response");
  }
  return await response.Body.transformToByteArray();
}

async function deleteObject(key: string) {
  const command = new DeleteObjectCommand({
    Bucket: BUCKET,
    Key: key,
  });
  await s3Client.send(command);
}

export default {
  put,
  get,
  deleteObject,
};
