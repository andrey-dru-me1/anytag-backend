// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use diesel::prelude::*;

use super::{PostId, TagId, UserId};

#[derive(Queryable, Identifiable, Debug, Clone)]
#[diesel(table_name = crate::schema::post_tags)]
#[diesel(primary_key(post_id, tag_id))]
pub struct PostTag {
    pub post_id: PostId,
    pub tag_id: TagId,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::post_tags)]
pub struct NewPostTag {
    pub post_id: PostId,
    pub tag_id: TagId,
}

#[derive(Queryable, Identifiable, Debug, Clone)]
#[diesel(table_name = crate::schema::user_tag_subscriptions)]
#[diesel(primary_key(user_id, tag_id))]
pub struct UserTagSubscription {
    pub user_id: UserId,
    pub tag_id: TagId,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::user_tag_subscriptions)]
pub struct NewUserTagSubscription {
    pub user_id: UserId,
    pub tag_id: TagId,
}

#[derive(Queryable, Identifiable, Debug, Clone)]
#[diesel(table_name = crate::schema::user_user_subscriptions)]
#[diesel(primary_key(follower_id, followed_id))]
pub struct UserUserSubscription {
    pub follower_id: UserId,
    pub followed_id: UserId,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::user_user_subscriptions)]
pub struct NewUserUserSubscription {
    pub follower_id: UserId,
    pub followed_id: UserId,
}

#[derive(Queryable, Identifiable, Debug, Clone)]
#[diesel(table_name = crate::schema::tag_user_visibility)]
#[diesel(primary_key(tag_id, user_id))]
pub struct TagUserVisibility {
    pub tag_id: TagId,
    pub user_id: UserId,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::tag_user_visibility)]
pub struct NewTagUserVisibility {
    pub tag_id: TagId,
    pub user_id: UserId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_tag_struct() {
        let post_tag = PostTag {
            post_id: 1,
            tag_id: 2,
        };

        assert_eq!(post_tag.post_id, 1);
        assert_eq!(post_tag.tag_id, 2);
    }

    #[test]
    fn test_user_tag_subscription_struct() {
        let subscription = UserTagSubscription {
            user_id: 1,
            tag_id: 2,
        };

        assert_eq!(subscription.user_id, 1);
        assert_eq!(subscription.tag_id, 2);
    }

    #[test]
    fn test_user_user_subscription_struct() {
        let subscription = UserUserSubscription {
            follower_id: 1,
            followed_id: 2,
        };

        assert_eq!(subscription.follower_id, 1);
        assert_eq!(subscription.followed_id, 2);
    }

    #[test]
    fn test_tag_user_visibility_struct() {
        let visibility = TagUserVisibility {
            tag_id: 1,
            user_id: 2,
        };

        assert_eq!(visibility.tag_id, 1);
        assert_eq!(visibility.user_id, 2);
    }
}
