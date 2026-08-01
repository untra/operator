//! CI gate for community-contributed collections.
//!
//! Every directory under `collections/community/` must be a valid shareable
//! collection: correct manifest, community-tier attribution (license, author,
//! url), and parseable issuetype schemas. A broken submission fails here,
//! never at a user's install.

use std::path::PathBuf;

use operator::collections::manifest::CollectionTier;
use operator::collections::validate::validate_collection_dir;

fn community_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("collections")
        .join("community")
}

#[test]
fn test_all_community_collections_are_valid() {
    let dir = community_dir();
    if !dir.is_dir() {
        // Green when the community tier is empty.
        return;
    }

    let mut validated = 0usize;
    for entry in std::fs::read_dir(&dir).expect("read collections/community") {
        let path = entry.expect("dir entry").path();
        if !path.is_dir() {
            continue;
        }
        let manifest = validate_collection_dir(&path)
            .unwrap_or_else(|e| panic!("{} is invalid: {e:#}", path.display()));
        assert_eq!(
            manifest.tier,
            CollectionTier::Community,
            "{}: community-hosted collections must declare tier: community",
            manifest.id
        );
        validated += 1;
    }

    println!("validated {validated} community collection(s)");
}
