ALTER TABLE sources ADD COLUMN reference TEXT;
ALTER TABLE sources ADD COLUMN effective_date INTEGER;
ALTER TABLE sources ADD COLUMN review_date INTEGER;
ALTER TABLE sources ADD COLUMN superseded_by TEXT;

ALTER TABLE resolutions ADD COLUMN context TEXT;
ALTER TABLE resolutions ADD COLUMN enforcement TEXT;
ALTER TABLE resolutions ADD COLUMN confidence REAL;
ALTER TABLE resolutions ADD COLUMN inputs TEXT NOT NULL DEFAULT '[]';
ALTER TABLE resolutions ADD COLUMN made_by TEXT;
ALTER TABLE resolutions ADD COLUMN approved_by TEXT;
ALTER TABLE resolutions ADD COLUMN approved_at INTEGER;
ALTER TABLE resolutions ADD COLUMN superseded_by TEXT;
