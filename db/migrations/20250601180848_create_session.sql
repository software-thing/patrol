-- migrate:up

CREATE TABLE IF NOT EXISTS "sessions" (
    "id" TEXT PRIMARY KEY,

    "user_username" TEXT NOT NULL REFERENCES "users"("username") ON DELETE CASCADE,

    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- migrate:down

DROP TABLE IF EXISTS "sessions";
