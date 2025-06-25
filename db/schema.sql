CREATE TABLE IF NOT EXISTS "schema_migrations" (version varchar(128) primary key);
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
CREATE TABLE IF NOT EXISTS "roles" (
  "title" TEXT PRIMARY KEY
);
CREATE TABLE IF NOT EXISTS "users_roles" (
  "user_username" TEXT NOT NULL REFERENCES "users"("username") ON DELETE CASCADE,
  "role_title" TEXT NOT NULL REFERENCES "roles"("title") ON DELETE CASCADE,

  "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  PRIMARY KEY ("user_username", "role_title")
);
CREATE INDEX "users_roles_user_username" ON "users_roles"("user_username");
CREATE INDEX "users_roles_role_title" ON "users_roles"("role_title");
CREATE TABLE IF NOT EXISTS "sessions" (
    "id" TEXT PRIMARY KEY,

    "user_username" TEXT NOT NULL REFERENCES "users"("username"),

    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- Dbmate schema migrations
INSERT INTO "schema_migrations" (version) VALUES
  ('20250109201731'),
  ('20250111183444'),
  ('20250601180848');
