mod common;

use post::service;

#[tokio::test]
async fn post_vote_toggle() {
    let pool = common::get_db_pool().await;
    let (_topic, post_slug, _) = common::seed_minimal(&pool).await;

    let post_id = service::resolve_post_b62(&pool, &post_slug)
        .await
        .expect("resolve post");

    // Upvote
    let count = service::cast_post_vote(&pool, 300, post_id, 1)
        .await
        .expect("upvote");
    assert_eq!(count, 1);

    // Downvote (changes vote)
    let count = service::cast_post_vote(&pool, 300, post_id, -1)
        .await
        .expect("downvote");
    assert_eq!(count, -1);

    // Remove vote
    let count = service::cast_post_vote(&pool, 300, post_id, 0)
        .await
        .expect("remove vote");
    assert_eq!(count, 0);
}

#[tokio::test]
async fn post_vote_multiple_users() {
    let pool = common::get_db_pool().await;
    let (_topic, post_slug, _) = common::seed_minimal(&pool).await;

    let post_id = service::resolve_post_b62(&pool, &post_slug)
        .await
        .expect("resolve post");

    service::cast_post_vote(&pool, 301, post_id, 1)
        .await
        .unwrap();
    service::cast_post_vote(&pool, 302, post_id, 1)
        .await
        .unwrap();
    service::cast_post_vote(&pool, 303, post_id, -1)
        .await
        .unwrap();

    let post = service::get_post(&pool, &post_slug, None)
        .await
        .expect("get post");
    assert_eq!(post.vote_count, 1); // 2 up - 1 down
}

#[tokio::test]
async fn comment_vote_consistency() {
    let pool = common::get_db_pool().await;
    let (topic, post_slug, comment_hash) = common::seed_minimal(&pool).await;

    let comment_id = service::resolve_comment_id(&pool, &comment_hash)
        .await
        .expect("resolve comment");

    service::cast_comment_vote(&pool, 400, comment_id, 1)
        .await
        .unwrap();
    service::cast_comment_vote(&pool, 401, comment_id, 1)
        .await
        .unwrap();
    service::cast_comment_vote(&pool, 402, comment_id, -1)
        .await
        .unwrap();

    let result = service::get_all_comments(&pool, &topic, &post_slug, None, 25, 0, "hot")
        .await
        .expect("get_all_comments");
    assert_eq!(result.data[0].vote_count, 1);
}

#[tokio::test]
async fn invalid_vote_direction() {
    let pool = common::get_db_pool().await;
    let (_topic, post_slug, _) = common::seed_minimal(&pool).await;

    let post_id = service::resolve_post_b62(&pool, &post_slug)
        .await
        .expect("resolve post");

    // direction 2 is clamped to 1 in the code, so it should "succeed"
    // But direction -5 should become -1
    let count = service::cast_post_vote(&pool, 500, post_id, -5)
        .await
        .expect("clamped downvote");
    assert_eq!(count, -1);
}
