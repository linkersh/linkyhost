CREATE TYPE "public"."oauth_type" AS ENUM('github');--> statement-breakpoint
CREATE TABLE "users" (
	"id" bigint PRIMARY KEY NOT NULL,
	"username" varchar NOT NULL,
	"email" varchar NOT NULL,
	"identifier" varchar NOT NULL,
	"oauth_type" "oauth_type" NOT NULL
);
