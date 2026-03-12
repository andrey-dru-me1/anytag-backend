// @generated automatically by Diesel CLI.

diesel::table! {
    post_tags (post_id, tag_id) {
        post_id -> Int4,
        tag_id -> Int4,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        user_id -> Int4,
        text -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    tag_user_visibility (tag_id, user_id) {
        tag_id -> Int4,
        user_id -> Int4,
    }
}

diesel::table! {
    tags (id) {
        id -> Int4,
        user_id -> Int4,
        label -> Varchar,
        public -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    user_tag_subscriptions (user_id, tag_id) {
        user_id -> Int4,
        tag_id -> Int4,
    }
}

diesel::table! {
    user_user_subscriptions (follower_id, followed_id) {
        follower_id -> Int4,
        followed_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::joinable!(post_tags -> posts (post_id));
diesel::joinable!(post_tags -> tags (tag_id));
diesel::joinable!(posts -> users (user_id));
diesel::joinable!(tag_user_visibility -> tags (tag_id));
diesel::joinable!(tag_user_visibility -> users (user_id));
diesel::joinable!(tags -> users (user_id));
diesel::joinable!(user_tag_subscriptions -> tags (tag_id));
diesel::joinable!(user_tag_subscriptions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    post_tags,
    posts,
    tag_user_visibility,
    tags,
    user_tag_subscriptions,
    user_user_subscriptions,
    users,
);
