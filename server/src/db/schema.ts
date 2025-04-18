import {
  bigint,
  pgEnum,
  pgTable,
  timestamp,
  varchar,
  jsonb,
  text,
  integer,
  real,
} from "drizzle-orm/pg-core";

export const oauthType = pgEnum("oauth_type", ["github"]);

export const users = pgTable("users", {
  id: bigint("id", { mode: "bigint" }).notNull().primaryKey(),
  username: varchar("username").notNull(),
  email: varchar("email"),
  identifier: varchar("identifier").notNull(),
  oauthType: oauthType("oauth_type").notNull(),
});

export const sessions = pgTable("sessions", {
  id: varchar("id").primaryKey().notNull(),
  userId: bigint("user_id", { mode: "bigint" })
    .references(() => users.id)
    .notNull(),
  userAgent: varchar("user_agent"),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const fileType = pgEnum("file_type", ["image", "video", "audio"]);
export const processingStatus = pgEnum("processing_status", [
  "pending",
  "processing",
  "completed",
  "failed",
]);

export const files = pgTable("files", {
  id: bigint("id", { mode: "bigint" }).notNull().primaryKey(),
  userId: bigint("user_id", { mode: "bigint" })
    .references(() => users.id)
    .notNull(),
  albumId: bigint("album_id", { mode: "bigint" }),
  fileName: varchar("file_name").notNull(),
  fileType: fileType("file_type").notNull(),
  mimeType: varchar("mime_type").notNull(),
  size: bigint("size", { mode: "number" }).notNull(),
  s3Key: text("s3_key").notNull(),
  width: integer("width"),
  height: integer("height"),
  exifData: jsonb("exif_data"),
  duration: real("duration"),
  waveform: real("waveform").array(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});

export const albums = pgTable("albums", {
  id: bigint("id", { mode: "bigint" }).notNull().primaryKey(),
  userId: bigint("user_id", { mode: "bigint" })
    .references(() => users.id)
    .notNull(),
  name: varchar("name").notNull(),
  coverFileId: bigint("cover_file_id", { mode: "bigint" }).references(
    () => files.id
  ),
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});

export const albumFiles = pgTable("album_files", {
  albumId: bigint("album_id", { mode: "bigint" })
    .references(() => albums.id)
    .notNull(),
  fileId: bigint("file_id", { mode: "bigint" })
    .references(() => files.id)
    .notNull(),
  orderIndex: integer("order_index").notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const processingQueue = pgTable("processing_queue", {
  id: bigint("id", { mode: "bigint" }).notNull().primaryKey(),
  fileId: bigint("file_id", { mode: "bigint" })
    .references(() => files.id)
    .notNull(),
  status: processingStatus("status").default("pending").notNull(),
  attempts: integer("attempts").default(0).notNull(),
  error: text("error"),
  processingStartedAt: timestamp("processing_started_at"),
  processingCompletedAt: timestamp("processing_completed_at"),
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});

export const thumbnails = pgTable("thumbnails", {
  id: bigint("id", { mode: "bigint" }).notNull().primaryKey(),
  fileId: bigint("file_id", { mode: "bigint" })
    .references(() => files.id)
    .notNull(),
  width: integer("width").notNull(),
  height: integer("height").notNull(),
  size: bigint("size", { mode: "bigint" }).notNull(),
  path: text("path").notNull(),
  format: varchar("format").notNull(),
  quality: integer("quality"),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});
