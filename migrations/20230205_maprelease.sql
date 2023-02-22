INSERT IGNORE INTO maps
  (id, name, courses, validated, filesize, created_by, approved_by, created_on, updated_on)
VALUES
  (1215, "kz_apiisnotresponding", 2, true, 52344340,  27894113,   241226935, "2023-02-05 20:00:28", "2023-02-05 20:00:28"),
  (1216, "kz_cf_slide",           4, true, 41263092,  351127846,  241226935, "2023-02-05 20:00:28", "2023-02-05 20:00:28"),
  (1217, "kz_desolate",           3, true, 75278696,  1107436699, 241226935, "2023-02-05 20:00:28", "2023-02-05 20:00:28"),
  (1218, "kz_itz_updown",         1, true, 140470496, 144500068,  241226935, "2023-02-05 20:00:28", "2023-02-05 20:00:28"),
  (1219, "kz_tangent",            1, true, 172930632, 1107436699, 241226935, "2023-02-05 20:00:28", "2023-02-05 20:00:28"),
  (1220, "kz_tense",              1, true, 37122636,  351127846,  241226935, "2023-02-05 20:00:28", "2023-02-05 20:00:28");

INSERT IGNORE INTO courses
  (id, map_id, stage, kzt, kzt_difficulty, skz, skz_difficulty, vnl, vnl_difficulty)
VALUES
  (121500, 1215, 0, true, 1, true, 1, true,  1),
  (121501, 1215, 1, true, 1, true, 1, true,  1),
  (121600, 1216, 0, true, 3, true, 3, false, 3),
  (121601, 1216, 1, true, 3, true, 3, false, 3),
  (121602, 1216, 2, true, 3, true, 3, false, 3),
  (121603, 1216, 3, true, 3, true, 3, false, 3),
  (121700, 1217, 0, true, 2, true, 2, true,  2),
  (121701, 1217, 1, true, 2, true, 2, true,  2),
  (121702, 1217, 2, true, 2, true, 2, true,  2),
  (121800, 1218, 0, true, 4, true, 4, false, 4),
  (121900, 1219, 0, true, 3, true, 3, false, 3),
  (122000, 1220, 0, true, 7, true, 7, false, 7);

UPDATE maps
SET name = "kz_acores", filesize = 131979212, updated_on = "2023-02-05 20:00:28"
WHERE name = "kz_alleviate";
