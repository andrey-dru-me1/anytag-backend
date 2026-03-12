CREATE TABLE tag_user_visibility (
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (tag_id, user_id)
);

CREATE INDEX tag_user_visibility_tag_id_idx ON tag_user_visibility(tag_id);
CREATE INDEX tag_user_visibility_user_id_idx ON tag_user_visibility(user_id);
