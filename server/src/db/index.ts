import { drizzle } from "drizzle-orm/node-postgres";
import * as schema from "@/db/schema";

export const DATABASE_URL = Bun.env.DATABASE_URL!;
export const db = drizzle(DATABASE_URL, { schema });

export * from "@/db/schema";
