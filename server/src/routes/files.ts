import { generateID, isAudio, isImage } from "@/utils";
import { middleware } from "./middleware";
import { db } from "@/db";
import { FileRepository } from "@/repos/files";
import Elysia, { error, t } from "elysia";
import sharp from "sharp";
import store from "@/store";

const APIFileSchema = t.Object({
  id: t.BigInt(),
  userId: t.BigInt(),
  albumId: t.Nullable(t.BigInt()),
  fileName: t.String(),
  fileType: t.String(),
  mimeType: t.String(),
  size: t.Number(),
  s3Key: t.String(),
  width: t.Nullable(t.Number()),
  height: t.Nullable(t.Number()),
  exifData: t.Nullable(t.Any()),
  duration: t.Nullable(t.Number()),
  waveform: t.Nullable(t.Array(t.Number())),
  createdAt: t.Date(),
  updatedAt: t.Date(),
});

export function filesRouter() {
  return new Elysia({ prefix: "/files" })
    .use(middleware())
    .get(
      "/:id/view",
      async ({ params: { id }, session }) => {
        const fileRepo = new FileRepository();

        const dbFile = await fileRepo.getFile({
          id: BigInt(id),
          userId: session.userId,
        });

        if (!dbFile) {
          throw error(404, "File not found");
        }

        const file = await store.get("image", BigInt(id));
        const blob = new Blob([file]);
        return new File([blob], dbFile.fileName, { type: dbFile.mimeType });
      },
      {
        params: t.Object({
          id: t.String(),
        }),
        response: t.File(),
      }
    )
    .post(
      "/upload",
      async ({ body: { file, data }, session }) => {
        if (isImage(file.type)) {
          const bytes = await file.bytes();
          const image = sharp(bytes);
          const metadata = await image.metadata();
          if (!metadata.width || !metadata.height) {
            throw error(400, "Invalid image");
          }

          const { dbFile, s3File } = await db.transaction(async (tx) => {
            const fileId = generateID();
            const s3File = await store.put("image", fileId, file.type, bytes);

            const fileRepo = new FileRepository(tx);
            const randomDate = new Date(
              new Date().getFullYear() - Math.floor(Math.random() * 5), // Random year within the last 5 years
              Math.floor(Math.random() * 10), // Random month
              Math.floor(Math.random() * 28) + 1 // Random day (1-28 to avoid invalid dates)
            );

            const dbFile = await fileRepo.createFile({
              id: fileId,
              type: "image",
              file: {
                name: file.name,
                contentType: file.type,
              },
              userId: session.userId,
              albumId: data.albumId ?? null,
              size: bytes.length,
              s3Key: s3File.key,
              createdAt: randomDate,
              width: metadata.width!,
              height: metadata.height!,
            });

            return { dbFile, s3File };
          });

          s3File.mark();
          return dbFile;
        } else if (isAudio(file.type)) {
          throw error(500, "not implemented");
        } else {
          throw error(400, "Unsupported file type");
        }
      },
      {
        body: t.Object({
          file: t.File(),
          data: t.ObjectString({
            albumId: t.Optional(t.BigInt()),
          }),
        }),
        response: APIFileSchema,
      }
    )
    .get(
      "/buckets",
      async ({ query: { type }, session }) => {
        const fileRepo = new FileRepository();
        const buckets = await fileRepo.getTimeBuckets({
          type: type,
          userId: session.userId,
        });
        return buckets.map((x) => ({ date: new Date(x.date), count: x.count }));
      },
      {
        query: t.Object({
          type: t.Union([
            t.Literal("image"),
            t.Literal("video"),
            t.Literal("audio"),
          ]),
        }),
        response: t.Array(
          t.Object({
            date: t.Date(),
            count: t.Integer(),
          })
        ),
      }
    )
    .get(
      "/buckets/files",
      async ({ query: { type, date }, session }) => {
        const fileRepo = new FileRepository();
        const files = await fileRepo.getBucketFiles({
          type: type,
          date: date,
          userId: session.userId,
        });
        return files;
      },
      {
        query: t.Object({
          type: t.Union([
            t.Literal("image"),
            t.Literal("video"),
            t.Literal("audio"),
          ]),
          date: t.String(),
        }),
        response: t.Array(APIFileSchema),
      }
    );
}
