-- migrate:up

CREATE TABLE IF NOT EXISTS "sessions" (
  "jti" UUID PRIMARY KEY,

  "sub" UUID NOT NULL REFERENCES "users"("id") ON DELETE CASCADE,

  "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS "users_sessions" (
  "user_id" UUID NOT NULL REFERENCES "users"("id") ON DELETE CASCADE,
  "session_jti" UUID NOT NULL REFERENCES "sessions"("jti") ON DELETE CASCADE,

  PRIMARY KEY ("user_id", "session_jti")
);

CREATE INDEX IF NOT EXISTS "sessions_user_id" ON "sessions"("user_id");

-- migrate:down

DROP INDEX IF EXISTS "sessions_user_id";

DROP TABLE IF EXISTS "sessions";
