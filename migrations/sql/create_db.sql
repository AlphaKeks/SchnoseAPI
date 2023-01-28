CREATE TABLE
  IF NOT EXISTS modes (
    id TINYINT UNSIGNED NOT NULL,
    name VARCHAR(255) NOT NULL,
    name_short VARCHAR(255) NOT NULL,
    name_long VARCHAR(255) NOT NULL,
    created_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
  );

CREATE TABLE
  IF NOT EXISTS players (
    id BIGINT UNSIGNED,
    name VARCHAR(255) NOT NULL DEFAULT "unknown",
    is_banned BOOL NOT NULL DEFAULT FALSE,
    first_login DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    playtime TIME NOT NULL DEFAULT "00:00",
    PRIMARY KEY (id)
  );

CREATE TABLE
  IF NOT EXISTS maps (
    id SMALLINT UNSIGNED NOT NULL,
    name VARCHAR(255) NOT NULL,
    difficulty TINYINT UNSIGNED NOT NULL,
    validated BOOL NOT NULL DEFAULT FALSE,
    filesize BIGINT UNSIGNED NOT NULL,
    created_by BIGINT UNSIGNED,
    approved_by BIGINT UNSIGNED,
    created_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (created_by) REFERENCES players (id),
    FOREIGN KEY (approved_by) REFERENCES players (id),
    INDEX h (validated, created_by, approved_on)
  );

CREATE TABLE
  IF NOT EXISTS servers (
    id SMALLINT UNSIGNED NOT NULL,
    name VARCHAR(255) NOT NULL,
    owner_id BIGINT UNSIGNED,
    approved_by BIGINT UNSIGNED,
    approved_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (owner_id) REFERENCES players (id),
    FOREIGN KEY (approved_by) REFERENCES players (id),
    INDEX owner_index (owner_id)
  );

CREATE TABLE
  IF NOT EXISTS records (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    map_id SMALLINT UNSIGNED NOT NULL,
    mode_id TINYINT UNSIGNED NOT NULL,
    player_id BIGINT UNSIGNED NOT NULL,
    server_id SMALLINT UNSIGNED NOT NULL,
    stage TINYINT UNSIGNED NOT NULL,
    teleports INT UNSIGNED NOT NULL,
    time DOUBLE NOT NULL,
    created_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    global_id INT UNSIGNED NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (map_id) REFERENCES maps (id),
    FOREIGN KEY (mode_id) REFERENCES modes (id),
    FOREIGN KEY (player_id) REFERENCES players (id),
    FOREIGN KEY (server_id) REFERENCES servers (id),
    INDEX h (
      id,
      mode_id,
      player_id,
      stage,
      teleports,
      global_id
    ),
    INDEX recent_index (mode_id, created_on),
    INDEX recent_pb_index (player_id, mode_id, teleports, created_on),
    INDEX recent_wr_index (mode_id, time, teleports, created_on)
  );

CREATE TABLE
  IF NOT EXISTS player_stats_kzt_tp (
    player_id BIGINT UNSIGNED NOT NULL,
    points SMALLINT UNSIGNED NOT NULL,
    teleports BIGINT UNSIGNED NOT NULL,
    records SMALLINT UNSIGNED NOT NULL,
    world_records SMALLINT UNSIGNED NOT NULL,
    FOREIGN KEY (player_id) REFERENCES players (id)
  );

CREATE TABLE
  IF NOT EXISTS player_stats_kzt_pro (
    player_id BIGINT UNSIGNED NOT NULL,
    points SMALLINT UNSIGNED NOT NULL,
    records SMALLINT UNSIGNED NOT NULL,
    world_records SMALLINT UNSIGNED NOT NULL,
    FOREIGN KEY (player_id) REFERENCES players (id)
  );

CREATE TABLE
  IF NOT EXISTS player_stats_skz_tp (
    player_id BIGINT UNSIGNED NOT NULL,
    points SMALLINT UNSIGNED NOT NULL,
    teleports BIGINT UNSIGNED NOT NULL,
    records SMALLINT UNSIGNED NOT NULL,
    world_records SMALLINT UNSIGNED NOT NULL,
    FOREIGN KEY (player_id) REFERENCES players (id)
  );

CREATE TABLE
  IF NOT EXISTS player_stats_skz_pro (
    player_id BIGINT UNSIGNED NOT NULL,
    points SMALLINT UNSIGNED NOT NULL,
    records SMALLINT UNSIGNED NOT NULL,
    world_records SMALLINT UNSIGNED NOT NULL,
    FOREIGN KEY (player_id) REFERENCES players (id)
  );

CREATE TABLE
  IF NOT EXISTS player_stats_vnl_tp (
    player_id BIGINT UNSIGNED NOT NULL,
    points SMALLINT UNSIGNED NOT NULL,
    teleports BIGINT UNSIGNED NOT NULL,
    records SMALLINT UNSIGNED NOT NULL,
    world_records SMALLINT UNSIGNED NOT NULL,
    FOREIGN KEY (player_id) REFERENCES players (id)
  );

CREATE TABLE
  IF NOT EXISTS player_stats_vnl_pro (
    player_id BIGINT UNSIGNED NOT NULL,
    points SMALLINT UNSIGNED NOT NULL,
    records SMALLINT UNSIGNED NOT NULL,
    world_records SMALLINT UNSIGNED NOT NULL,
    FOREIGN KEY (player_id) REFERENCES players (id)
  );
