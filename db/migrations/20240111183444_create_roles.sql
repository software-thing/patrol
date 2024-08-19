-- migrate:up

CREATE TABLE IF NOT EXISTS "roles" (
  "title" TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS "users_roles" (
  "user_username" VARCHAR(64) NOT NULL REFERENCES "users"("username") ON DELETE CASCADE,
  "role_title" TEXT NOT NULL REFERENCES "roles"("title") ON DELETE CASCADE,

  PRIMARY KEY ("user_username", "role_title")
);

CREATE INDEX IF NOT EXISTS "users_roles_user_username" ON "users_roles"("user_username");
CREATE INDEX IF NOT EXISTS "users_roles_role_title" ON "users_roles"("role_title");

INSERT INTO "roles" VALUES ('admin');

-- migrate:down

DROP INDEX IF EXISTS "users_roles_user_id";

DROP TABLE IF EXISTS "users_roles";

DROP TABLE IF EXISTS "roles";
