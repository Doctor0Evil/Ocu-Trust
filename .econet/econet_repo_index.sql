-- filename: .econet/econet_repo_index.sql
-- Purpose: Master index SQL-shard for econet constellation repos.
-- Guides AI chat and coding agents on repo roles, programming layers, and safety invariants.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS econet_repo_index (
    repo_name           TEXT PRIMARY KEY,
    github_slug         TEXT NOT NULL,
    role_band           TEXT NOT NULL, -- 'SPINE','RESEARCH','ENGINE','MATERIAL','GOV','APP'
    visibility          TEXT NOT NULL, -- 'Public','Private'
    language_primary    TEXT NOT NULL, -- 'Rust','C++','C#','JS','HTML','Other'
    description         TEXT,
    ecosafety_binding   TEXT NOT NULL, -- e.g. 'cyboquatic-ecosafety-core=2026v1'
    shard_protocol      TEXT NOT NULL, -- e.g. 'ALN-RFC4180/EcoNetSchemaShard2026v1'
    lane_default        TEXT NOT NULL, -- 'RESEARCH','EXPPROD','PROD'
    ker_target_k        REAL NOT NULL,
    ker_target_e        REAL NOT NULL,
    ker_target_r        REAL NOT NULL,
    non_actuating_only  INTEGER NOT NULL CHECK (non_actuating_only IN (0,1))
);

CREATE TABLE IF NOT EXISTS econet_layer (
    layer_id            INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_name           TEXT NOT NULL REFERENCES econet_repo_index(repo_name) ON DELETE CASCADE,
    layer_name          TEXT NOT NULL,
    layer_tier          TEXT NOT NULL, -- 'GRAMMAR','KERNEL','EDGE_SCRIPT','UI','GOVERNANCE','MATERIAL','OTHER'
    languages           TEXT NOT NULL, -- comma-separated, e.g. 'Rust', 'C++,C'
    description         TEXT,
    contracts           TEXT           -- human-readable invariants (no corridor, no build; Vt<=; etc.)
);

CREATE TABLE IF NOT EXISTS econet_role_hint (
    hint_id             INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_name           TEXT NOT NULL REFERENCES econet_repo_index(repo_name) ON DELETE CASCADE,
    key                 TEXT NOT NULL, -- e.g. 'shard_types','primary_particles','pilot_domains'
    value               TEXT NOT NULL  -- e.g. 'PhoenixMarShard,HydrologicalBufferShard'
);

-- Example insert for EcoNet-CEIM-PhoenixWater.
-- For other repos, duplicate this block and adjust values.

INSERT OR REPLACE INTO econet_repo_index
(repo_name, github_slug, role_band, visibility, language_primary,
 description, ecosafety_binding, shard_protocol, lane_default,
 ker_target_k, ker_target_e, ker_target_r, non_actuating_only)
VALUES
(
  'EcoNet-CEIM-PhoenixWater',
  'Doctor0Evil/EcoNet-CEIM-PhoenixWater',
  'ENGINE',
  'Public',
  'Rust',
  'Phoenix water CEIM/CPVM kernels and ecosafety-controlled controllers for PFBS, E. coli, salinity, hydrological buffers.',
  'cyboquatic-ecosafety-core=EcosafetyGrammar2026v1.aln',
  'ALN-RFC4180/EcoNetSchemaShard2026v1',
  'PROD',
  0.94, 0.90, 0.12,
  0
);

INSERT INTO econet_layer
(repo_name, layer_name, layer_tier, languages, description, contracts)
VALUES
(
  'EcoNet-CEIM-PhoenixWater',
  'Ecosafety spine client',
  'GRAMMAR',
  'Rust',
  'Links to cyboquatic-ecosafety-core RiskCoord,RiskVector,Residual, KER; no custom Lyapunov norms.',
  'Must import EcosafetyGrammar2026v1; no new risk planes; Vt_next <= Vt; no corridor, no build; violated corridor => derate/stop.'
),
(
  'EcoNet-CEIM-PhoenixWater',
  'CEIM/CPVM hydrology kernels',
  'KERNEL',
  'Rust,C',
  'Implements CEIM mass-load Mx and CPVM viability V for Phoenix basins using ALN qpudatashards.',
  'No actuation ownership; kernels purely compute Mx, Kn, rx, Vt, KER from qpudatashards; must pass csvlint and schema checks.'
),
(
  'EcoNet-CEIM-PhoenixWater',
  'Hydrological buffer atlas',
  'KERNEL',
  'Rust',
  'Non-actuating atlas that maintains rFOG, rTDS, rEcoli, Vt for reaches using HydrologicalBufferPhoenix shards.',
  'No actuator handles; only shard read/write; Vt monotone, corridors only tighten; outputs lane-tagged RESEARCH/PROD.'
);

INSERT INTO econet_role_hint
(repo_name, key, value)
VALUES
('EcoNet-CEIM-PhoenixWater', 'shard_types', 'PhoenixMarShard,HydrologicalBufferShard,FOGRoutingDecision'),
('EcoNet-CEIM-PhoenixWater', 'primary_particles', 'HydrologicalBufferPhoenix2026v1, PhoenixMarShard.v2'),
('EcoNet-CEIM-PhoenixWater', 'pilot_domains', 'Central-AZ water remediation, Gila E. coli buffering, Lake Pleasant PFBS monitoring');
