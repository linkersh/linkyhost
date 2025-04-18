CREATE TABLE "album_files" (
	"album_id" bigint NOT NULL,
	"file_id" bigint NOT NULL,
	"order_index" integer NOT NULL,
	"created_at" timestamp DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "albums" (
	"id" bigint PRIMARY KEY NOT NULL,
	"user_id" bigint NOT NULL,
	"name" varchar NOT NULL,
	"cover_file_id" bigint,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"updated_at" timestamp DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "files" ALTER COLUMN "file_type" SET DATA TYPE text;--> statement-breakpoint
DROP TYPE "public"."file_type";--> statement-breakpoint
CREATE TYPE "public"."file_type" AS ENUM('image', 'video', 'audio');--> statement-breakpoint
ALTER TABLE "files" ALTER COLUMN "file_type" SET DATA TYPE "public"."file_type" USING "file_type"::"public"."file_type";--> statement-breakpoint
ALTER TABLE "files" ADD COLUMN "album_id" bigint;--> statement-breakpoint
ALTER TABLE "files" ADD COLUMN "s3_key" text NOT NULL;--> statement-breakpoint
ALTER TABLE "files" ADD COLUMN "width" integer;--> statement-breakpoint
ALTER TABLE "files" ADD COLUMN "height" integer;--> statement-breakpoint
ALTER TABLE "files" ADD COLUMN "duration" real;--> statement-breakpoint
ALTER TABLE "files" ADD COLUMN "waveform" real[];--> statement-breakpoint
ALTER TABLE "album_files" ADD CONSTRAINT "album_files_album_id_albums_id_fk" FOREIGN KEY ("album_id") REFERENCES "public"."albums"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "album_files" ADD CONSTRAINT "album_files_file_id_files_id_fk" FOREIGN KEY ("file_id") REFERENCES "public"."files"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "albums" ADD CONSTRAINT "albums_user_id_users_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."users"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "albums" ADD CONSTRAINT "albums_cover_file_id_files_id_fk" FOREIGN KEY ("cover_file_id") REFERENCES "public"."files"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "files" DROP COLUMN "path";