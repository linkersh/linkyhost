import { getGithubProfile, JWT_EXP, UserRepository } from "@/repos/user";
import { oauth2 } from "elysia-oauth2";
import Elysia, { redirect } from "elysia";
import { db } from "@/db";

export function authRouter() {
  return new Elysia({ prefix: "/auth" })
    .use(
      oauth2({
        GitHub: [
          Bun.env.GITHUB_CLIENT_ID!,
          Bun.env.GITHUB_CLIENT_SECRET!,
          `http://${Bun.env.FRONTEND_URL}/oauth/github`,
        ],
      })
    )
    .get("/github", async ({ oauth2 }) =>
      oauth2.redirect("GitHub", ["user:email"])
    )
    .get("/github/callback", async ({ oauth2, cookie: { __sessionToken } }) => {
      const tokens = await oauth2.authorize("GitHub");
      const accessToken = tokens.accessToken();

      const profile = await getGithubProfile(accessToken);
      const token = await db.transaction(async (tx) => {
        const userRepo = new UserRepository(tx);
        const user = await userRepo.createUser({
          githubId: profile.id,
          login: profile.login,
          email: profile.email,
        });

        const token = await userRepo.createSession(user.id, null);
        return token;
      });

      __sessionToken!.set({
        value: token,
        domain: Bun.env.SERVER_URL,
        expires: new Date(Date.now() + JWT_EXP),
        httpOnly: true,
      });

      return redirect(`http://${Bun.env.FRONTEND_URL}/dash/photos`);
    });
}
