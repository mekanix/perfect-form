CREATE TABLE "roles" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_roles_by_name" ON "roles" ("name");
-- #[toasty::breakpoint]
CREATE TABLE "users_roles" (
    "user_id" INTEGER NOT NULL,
    "role_id" INTEGER NOT NULL,
    PRIMARY KEY ("user_id", "role_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_users_roles_by_user_id" ON "users_roles" ("user_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_users_roles_by_role_id" ON "users_roles" ("role_id");
-- #[toasty::breakpoint]
CREATE TABLE "users" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "email" TEXT NOT NULL,
    "password_hash" TEXT NOT NULL,
    "admin" BOOLEAN NOT NULL
);
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_users_by_email" ON "users" ("email");
