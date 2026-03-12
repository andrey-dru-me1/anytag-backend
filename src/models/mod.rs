// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

// Type aliases for entity IDs
pub type Id = i32;
pub type UserId = Id;
pub type PostId = Id;
pub type TagId = Id;

mod post;
mod relations;
mod tag;
mod user;

pub use post::*;
pub use tag::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_aliases() {
        // Verify type aliases are correct
        let _id: Id = 42;
        let _user_id: UserId = 42;
        let _post_id: PostId = 42;
        let _tag_id: TagId = 42;

        // This just compiles, which is the test
        assert!(true);
    }
}
