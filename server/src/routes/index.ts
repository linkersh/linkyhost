import Elysia from "elysia";
import { filesRouter } from "@/routes/files";
import { authRouter } from "./auth";
import { middleware } from "./middleware";

export function apiRouter() {
  return new Elysia({ prefix: "/api" }).use(authRouter()).use(filesRouter());
}
