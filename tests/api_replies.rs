mod common;

use post::service;

#[tokio::test]
async fn create_and_get_replies() {
    let pool = common::get_db_pool().await;
    let (topic, post_slug, comment_hash) = common::seed_minimal(&pool).await;

    // Create a top-level reply
    service::create_reply(
        &pool,
        102,
        &topic,
        &post_slug,
        &comment_hash,
        "First reply",
        None,
    )
    .await
    .expect("create first reply");

    let result = service::get_replies(&pool, &topic, &post_slug, &comment_hash, None, 25, 0)
        .await
        .expect("get_replies should succeed");

    assert_eq!(result.total, Some(1));
    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].content, "First reply");
    assert!(result.data[0].children.is_empty());
}

#[tokio::test]
async fn replies_are_hierarchical() {
    let pool = common::get_db_pool().await;
    let (topic, post_slug, comment_hash) = common::seed_minimal(&pool).await;

    // Create reply A (top-level)
    service::create_reply(
        &pool,
        102,
        &topic,
        &post_slug,
        &comment_hash,
        "Reply A",
        None,
    )
    .await
    .expect("create reply A");

    // Get A's hash
    let result = service::get_replies(&pool, &topic, &post_slug, &comment_hash, None, 25, 0)
        .await
        .expect("get_replies");
    let hash_a = &result.data[0].hash;

    // Create reply B as child of A
    service::create_reply(
        &pool,
        103,
        &topic,
        &post_slug,
        &comment_hash,
        "Reply B (child of A)",
        Some(hash_a),
    )
    .await
    .expect("create reply B");

    // Create reply C as child of B
    let result = service::get_replies(&pool, &topic, &post_slug, &comment_hash, None, 25, 0)
        .await
        .expect("get_replies");
    let hash_b = &result.data[0].children[0].hash;

    service::create_reply(
        &pool,
        104,
        &topic,
        &post_slug,
        &comment_hash,
        "Reply C (child of B)",
        Some(hash_b),
    )
    .await
    .expect("create reply C");

    // Verify the full tree
    let result = service::get_replies(&pool, &topic, &post_slug, &comment_hash, None, 25, 0)
        .await
        .expect("get_replies");

    assert_eq!(result.data.len(), 1); // 1 top-level: A
    assert_eq!(result.data[0].children.len(), 1); // 1 child: B
    assert_eq!(result.data[0].children[0].content, "Reply B (child of A)");
    assert_eq!(result.data[0].children[0].children.len(), 1); // 1 grandchild: C
    assert_eq!(
        result.data[0].children[0].children[0].content,
        "Reply C (child of B)"
    );
    assert!(result.data[0].children[0].children[0].children.is_empty());
}

#[tokio::test]
async fn replies_pagination() {
    let pool = common::get_db_pool().await;
    let (topic, post_slug, comment_hash) = common::seed_minimal(&pool).await;

    let _topic_id: i64 = sqlx::query_scalar("SELECT id FROM topics WHERE name = $1")
        .bind(&topic)
        .fetch_one(&pool)
        .await
        .expect("get topic id");

    let post_id: i64 = sqlx::query_scalar("SELECT id FROM posts WHERE slug = $1")
        .bind(&post_slug)
        .fetch_one(&pool)
        .await
        .expect("get post id");

    let comment_id: i64 = sqlx::query_scalar("SELECT id FROM comments WHERE hash = $1")
        .bind(&comment_hash)
        .fetch_one(&pool)
        .await
        .expect("get comment id");

    // Create 5 top-level replies
    for i in 0..5 {
        let hash = format!("r{}", (200 + i));
        sqlx::query(
            "INSERT INTO replies (hash, sender_id, post_id, comment_id, content)
             VALUES ($1, 110, $2, $3, $4)",
        )
        .bind(&hash)
        .bind(post_id)
        .bind(comment_id)
        .bind(format!("Top-level reply {}", i))
        .execute(&pool)
        .await
        .expect("seed reply");
    }

    // Page 1 (limit 3)
    let page1 = service::get_replies(&pool, &topic, &post_slug, &comment_hash, None, 3, 0)
        .await
        .expect("page1");
    assert_eq!(page1.data.len(), 3);
    assert_eq!(page1.total, Some(5));

    // Page 2 (limit 3, offset 3)
    let page2 = service::get_replies(&pool, &topic, &post_slug, &comment_hash, None, 3, 3)
        .await
        .expect("page2");
    assert_eq!(page2.data.len(), 2);
}

#[tokio::test]
async fn replies_vote_count() {
    let pool = common::get_db_pool().await;
    let (topic, post_slug, comment_hash) = common::seed_minimal(&pool).await;

    let _topic_id: i64 = sqlx::query_scalar("SELECT id FROM topics WHERE name = $1")
        .bind(&topic)
        .fetch_one(&pool)
        .await
        .expect("get topic id");

    let post_id: i64 = sqlx::query_scalar("SELECT id FROM posts WHERE slug = $1")
        .bind(&post_slug)
        .fetch_one(&pool)
        .await
        .expect("get post id");

    let comment_id: i64 = sqlx::query_scalar("SELECT id FROM comments WHERE hash = $1")
        .bind(&comment_hash)
        .fetch_one(&pool)
        .await
        .expect("get comment id");

    // Create a reply and get its id
    let reply_id: i64 = sqlx::query_scalar(
        "INSERT INTO replies (hash, sender_id, post_id, comment_id, content)
         VALUES ('v1xyz', 110, $1, $2, 'Votable reply')
         RETURNING id",
    )
    .bind(post_id)
    .bind(comment_id)
    .fetch_one(&pool)
    .await
    .expect("seed reply");

    // Vote on it
    let count = service::cast_reply_vote(&pool, 200, reply_id, 1)
        .await
        .expect("upvote should succeed");
    assert_eq!(count, 1);

    let count = service::cast_reply_vote(&pool, 201, reply_id, 1)
        .await
        .expect("second upvote");
    assert_eq!(count, 2);

    let count = service::cast_reply_vote(&pool, 200, reply_id, 0)
        .await
        .expect("remove vote");
    assert_eq!(count, 1);

    // Verify via get_replies
    let result = service::get_replies(&pool, &topic, &post_slug, &comment_hash, None, 25, 0)
        .await
        .expect("get_replies");
    assert_eq!(result.data[0].vote_count, 1);
}

#[tokio::test]
async fn replies_empty_for_unknown_comment() {
    let pool = common::get_db_pool().await;
    let (topic, post_slug, _) = common::seed_minimal(&pool).await;

    let result = service::get_replies(&pool, &topic, &post_slug, "nonexistent", None, 25, 0)
        .await
        .expect("get_replies should succeed with empty result");
    assert_eq!(result.data.len(), 0);
    assert_eq!(result.total, Some(0));
}
