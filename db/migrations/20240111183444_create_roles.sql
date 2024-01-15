-- migrate:up

CREATE TABLE IF NOT EXISTS "roles" (
  "title" TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS "users_roles" (
  "user_id" UUID NOT NULL REFERENCES "users"("id") ON DELETE CASCADE,
  "role_title" TEXT NOT NULL REFERENCES "roles"("title") ON DELETE CASCADE,

  PRIMARY KEY ("user_id", "role_title")
);

CREATE INDEX IF NOT EXISTS "users_roles_user_id" ON "users_roles"("user_id");

INSERT INTO "roles"("title") VALUES ('admin');

-- migrate:down

DROP INDEX IF EXISTS "users_roles_user_id";

DROP TABLE IF EXISTS "users_roles";

DROP TABLE IF EXISTS "roles";
