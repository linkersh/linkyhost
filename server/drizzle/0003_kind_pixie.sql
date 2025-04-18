CREATE TABLE "thumbnails" (
	"id" bigint PRIMARY KEY NOT NULL,
	"file_id" bigint NOT NULL,
	"width" integer NOT NULL,
	"height" integer NOT NULL,
	"size" bigint NOT NULL,
	"path" text NOT NULL,
	"format" varchar NOT NULL,
	"quality" integer,
	"created_at" timestamp DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "processing_queue" ALTER COLUMN "attempts" SET DATA TYPE integer;--> statement-breakpoint
ALTER TABLE "thumbnails" ADD CONSTRAINT "thumbnails_file_id_files_id_fk" FOREIGN KEY ("file_id") REFERENCES "public"."files"("id") ON DELETE no action ON UPDATE no action;