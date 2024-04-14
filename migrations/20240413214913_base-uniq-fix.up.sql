BEGIN;

WITH max_upgrades AS (
  SELECT user_id, max(storage_level) AS max_storage_level
  FROM bases
  GROUP BY user_id
)
DELETE FROM bases
WHERE (user_id, storage_level) NOT IN (
  SELECT user_id, max_storage_level FROM max_upgrades
);

-- de-dupe
DELETE FROM bases
WHERE ctid NOT IN (
  SELECT min(ctid)
  FROM bases
  GROUP BY user_id, storage_level
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_user_id_unique_storage_level ON bases(user_id);

COMMIT;
