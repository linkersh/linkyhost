import { Snowflake } from "@sapphire/snowflake";
import { audioMimeTypes, EPOCH, imageMimeTypes } from "../const";
import type { ExtractTablesWithRelations } from "drizzle-orm";
import type { NodePgQueryResultHKT } from "drizzle-orm/node-postgres";
import type { PgTransaction } from "drizzle-orm/pg-core";

export type PgTrans = PgTransaction<
  NodePgQueryResultHKT,
  typeof import("../db/schema"),
  ExtractTablesWithRelations<typeof import("../db/schema")>
>;

const snowflake = new Snowflake(EPOCH);

export function generateID() {
  return snowflake.generate();
}

export function isImage(mimeType: string): boolean {
  return imageMimeTypes.some((x) => mimeType.startsWith(x));
}

export function isAudio(mimeType: string): boolean {
  return audioMimeTypes.some((x) => mimeType.startsWith(x));
}
