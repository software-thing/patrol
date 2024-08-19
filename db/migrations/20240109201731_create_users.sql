-- migrate:up

CREATE TABLE IF NOT EXISTS "users" (
  "username" VARCHAR(64) PRIMARY KEY,

  "first_name" VARCHAR(64) NOT NULL,
  "last_name" VARCHAR(64) NOT NULL,

  "password_hash" TEXT NOT NULL,
  "password_hash_previous" TEXT DEFAULT NULL,
  "password_changed_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  "profile_picture" BOOLEAN NOT NULL DEFAULT FALSE,

  "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- migrate:down

DROP TABLE IF EXISTS "users";
