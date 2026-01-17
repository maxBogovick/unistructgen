-- Schema Registry Database Schema

-- Teams table
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Schemas table
CREATE TABLE schemas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    format VARCHAR(50) NOT NULL,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    description TEXT,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255) NOT NULL,
    UNIQUE(name, version)
);

CREATE INDEX idx_schemas_name ON schemas(name);
CREATE INDEX idx_schemas_team ON schemas(team_id);
CREATE INDEX idx_schemas_created_at ON schemas(created_at DESC);

-- Schema versions metadata
CREATE TABLE schema_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schema_name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    is_breaking BOOLEAN NOT NULL DEFAULT false,
    changelog TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(schema_name, version)
);

CREATE INDEX idx_versions_schema ON schema_versions(schema_name);

-- Generated artifacts
CREATE TABLE generated_artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schema_id UUID NOT NULL REFERENCES schemas(id) ON DELETE CASCADE,
    target VARCHAR(50) NOT NULL,
    content BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_artifacts_schema ON generated_artifacts(schema_id);
CREATE INDEX idx_artifacts_target ON generated_artifacts(target);

-- Team members
CREATE TABLE team_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id VARCHAR(255) NOT NULL,
    permissions JSONB NOT NULL DEFAULT '[]'::jsonb,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(team_id, user_id)
);

CREATE INDEX idx_members_team ON team_members(team_id);
CREATE INDEX idx_members_user ON team_members(user_id);

-- Generation events (for analytics)
CREATE TABLE generation_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schema_id UUID NOT NULL REFERENCES schemas(id) ON DELETE CASCADE,
    target VARCHAR(50) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_events_schema ON generation_events(schema_id);
CREATE INDEX idx_events_created_at ON generation_events(created_at DESC);

-- Insert default team
INSERT INTO teams (name, description) VALUES
    ('default', 'Default team for schemas');
