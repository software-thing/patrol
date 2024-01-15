-- migrate:up

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "users" (
  "id" UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

  "username" TEXT NOT NULL UNIQUE,

  "first_name" TEXT NOT NULL,
  "last_name" TEXT NOT NULL,

  "password_hash" TEXT NOT NULL,
  "password_hash_previous" TEXT,
  "password_changed_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  "profile_picture" BOOLEAN NOT NULL DEFAULT FALSE,

  "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- migrate:down

DROP TABLE IF EXISTS "users";

DROP EXTENSION IF EXISTS "uuid-ossp";
