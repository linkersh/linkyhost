import { UserRepository } from "@/repos/user";
import Elysia, { error } from "elysia";

const userRepo = new UserRepository();
export function middleware() {
  return new Elysia({ name: "auth-middleware" })
    .derive(async ({ cookie: { __sessionToken } }) => {
      if (!__sessionToken || !__sessionToken.value) throw error(401);

      const session = await userRepo.verifySession(__sessionToken.value);
      if (!session) throw error(401);

      return { session };
    })
    .as("plugin");
}
