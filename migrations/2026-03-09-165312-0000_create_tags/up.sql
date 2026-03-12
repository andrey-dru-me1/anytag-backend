CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    label VARCHAR NOT NULL,
    public BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX tags_user_id_idx ON tags(user_id);
CREATE INDEX tags_public_idx ON tags(public) WHERE public = true;
