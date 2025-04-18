import { db, sessions, users } from "@/db";
import { generateID, type PgTrans } from "@/utils";
import { eq } from "drizzle-orm";
import { jwtVerify, SignJWT } from "jose";
import crypto from "node:crypto";

const JWT_SECRET_ENV = Bun.env.JWT_SECRET!;
const JWT_ISSUER = "linker.sh";
const JWT_SECRET = new TextEncoder().encode(JWT_SECRET_ENV);
export const JWT_EXP = 60 * 60 * 24 * 30 * 1000; // 30 days

export interface GithubProfile {
  id: number;
  login: string;
  email: string | null;
}

export async function getGithubProfile(
  accessToken: string
): Promise<GithubProfile> {
  const response = await fetch("https://api.github.com/user", {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
  const user = (await response.json()) as GithubProfile;
  return { id: user.id, login: user.login, email: user.email };
}

export class UserRepository {
  tx?: PgTrans;

  constructor(tx?: PgTrans) {
    this.tx = tx;
  }

  get exec() {
    return this.tx ?? db;
  }

  public async createUser({
    githubId,
    login,
    email,
  }: {
    githubId: number;
    login: string;
    email: string | null;
  }) {
    const id = generateID();
    const [user] = await this.exec
      .insert(users)
      .values({
        id,
        email,
        username: login,
        identifier: githubId.toString(),
        oauthType: "github",
      })
      .returning();
    return user!;
  }

  public async createSession(
    userId: bigint,
    userAgent: string | null
  ): Promise<string> {
    const id = generateSecret();
    const token = await new SignJWT()
      .setIssuer(JWT_ISSUER)
      .setExpirationTime(JWT_EXP)
      .setSubject(id)
      .setIssuedAt()
      .setProtectedHeader({ alg: "HS512" })
      .sign(JWT_SECRET);
    await this.exec.insert(sessions).values({
      id,
      userId,
      userAgent,
    });
    return token;
  }

  public async verifySession(
    token: string
  ): Promise<typeof sessions.$inferSelect | null> {
    const claims = await jwtVerify(token, JWT_SECRET, {
      algorithms: ["HS512"],
      issuer: JWT_ISSUER,
    });

    const sessionId = claims.payload.sub!;
    if (!sessionId) {
      return null;
    }

    const session = await this.exec.query.sessions.findFirst({
      where: eq(sessions.id, sessionId),
    });
    return session ?? null;
  }
}

function generateSecret(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(32));
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}
