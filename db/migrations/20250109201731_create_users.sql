-- migrate:up

CREATE TABLE IF NOT EXISTS "users" (
  "username" TEXT PRIMARY KEY,

  "first_name" TEXT NOT NULL,
  "last_name" TEXT NOT NULL,

  "password_hash" TEXT NOT NULL,
  "password_hash_previous" TEXT DEFAULT NULL,
  "password_changed_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  "profile_picture" BOOLEAN NOT NULL DEFAULT FALSE,

  "approved" BOOLEAN NOT NULL DEFAULT FALSE,

  "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "updated_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- migrate:down

DROP TABLE IF EXISTS "users";
