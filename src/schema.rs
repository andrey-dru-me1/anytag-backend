// @generated automatically by Diesel CLI.

diesel::table! {
    image_sources (file_sha256_hash) {
        #[max_length = 64]
        file_sha256_hash -> Bpchar,
        #[max_length = 512]
        s3_path -> Varchar,
        #[max_length = 15]
        extension -> Varchar,
        file_size -> Int8,
        #[max_length = 100]
        mime_type -> Varchar,
        #[max_length = 63]
        bucket_name -> Varchar,
        width -> Int4,
        height -> Int4,
    }
}

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
    user_images (id) {
        id -> Int4,
        #[max_length = 64]
        file_sha256_hash -> Bpchar,
        #[max_length = 255]
        original_file_name -> Varchar,
        created_by -> Int4,
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
diesel::joinable!(user_images -> image_sources (file_sha256_hash));
diesel::joinable!(user_images -> users (created_by));
diesel::joinable!(user_tag_subscriptions -> tags (tag_id));
diesel::joinable!(user_tag_subscriptions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    image_sources,
    post_tags,
    posts,
    tag_user_visibility,
    tags,
    user_images,
    user_tag_subscriptions,
    user_user_subscriptions,
    users,
);
