INSERT INTO maps
  (id, name, courses, validated, filesize, created_by, approved_by, created_on, updated_on)
VALUES
  (1222, "kz_silk", 1, true, 127188048, 885989794, 241226935, "2023-03-12T18:16:15", "2023-03-12T18:16:15"),
  (1223, "kz_sxb_xbcmzl", 1, true, 88496168, 346464810, 346464810, "2023-03-12T18:16:15", "2023-03-12T18:16:15"),
  (1224, "kz_sxb_makabaka", 1, true, 87228728, 202059736, 346464810, "2023-03-12T18:16:15", "2023-03-12T18:16:15"),
  (1225, "kz_sxb_biewan", 1, true, 102470276, 321627999, 346464810, "2023-03-12T18:16:15", "2023-03-12T18:16:15"),
  (1226, "kz_cf_hestia", 2, true, 100058628, 351127846, 346464810, "2023-03-12T18:16:15", "2023-03-12T18:16:15"),
  (1227, "kz_sxb_poi", 1, true, 87068068, 321627999, 346464810, "2023-03-12T18:16:15", "2023-03-12T18:16:15"),
  (1228, "kz_cf_foliage", 2, true, 76066352, 351127846, 346464810, "2023-03-12T18:16:15", "2023-03-12T18:16:15");

INSERT INTO courses
  (id, map_id, stage, kzt, kzt_difficulty, skz, skz_difficulty, vnl, vnl_difficulty)
VALUES
  (122200, 1222, 0, true, 2, true, 2, true, 2),
  (122300, 1223, 0, true, 6, true, 6, false, 6),
  (122400, 1224, 0, true, 6, true, 6, false, 6),
  (122500, 1225, 0, true, 5, true, 5, false, 5),
  (122600, 1226, 0, true, 3, true, 3, true, 3),
  (122601, 1226, 1, true, 3, true, 3, true, 3),
  (122700, 1227, 0, true, 4, true, 4, false, 4),
  (122800, 1228, 0, true, 7, true, 7, false, 7),
  (122801, 1228, 1, true, 7, true, 7, false, 7);

UPDATE maps
SET name = "kz_cybersand", updated_on = "2023-03-12T18:16:15", courses = 4
WHERE name = "kz_cyberspace_fix";
INSERT INTO courses
  (id, map_id, stage, kzt, kzt_difficulty, skz, skz_difficulty, vnl, vnl_difficulty)
VALUES
  (71203, 712, 3, true, 2, true, 2, true, 2);

UPDATE maps
SET name = "kz_gus_sct2", updated_on = "2023-03-12T18:16:15"
WHERE name = "kz_gus";
