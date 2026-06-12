mod common;

use post::service;

#[tokio::test]
async fn create_and_list_comments() {
    let pool = common::get_db_pool().await;
    let (topic, post_slug, _) = common::seed_minimal(&pool).await;

    let result = service::get_all_comments(&pool, &topic, &post_slug, None, 25, 0, "new")
        .await
        .expect("get_all_comments should succeed");
    assert_eq!(result.data.len(), 1);
    assert_eq!(result.total, Some(1));
    assert_eq!(result.limit, Some(25));
    assert_eq!(result.offset, Some(0));
    assert_eq!(result.data[0].content, "A test comment");
}

#[tokio::test]
async fn comments_pagination() {
    let pool = common::get_db_pool().await;
    let (topic, post_slug, _) = common::seed_minimal(&pool).await;

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

    // Insert more comments
    for i in 0..5 {
        let hash = format!("c{}", (100 + i));
        sqlx::query("INSERT INTO comments (hash, sender_id, post_id, content) VALUES ($1, 200, $2, $3)")
            .bind(&hash)
            .bind(post_id)
            .bind(format!("Comment {}", i))
            .execute(&pool)
            .await
            .expect("seed extra comment");
    }

    // Page 1 (limit 3)
    let page1 = service::get_all_comments(&pool, &topic, &post_slug, None, 3, 0, "new")
        .await
        .expect("page1");
    assert_eq!(page1.data.len(), 3);
    assert_eq!(page1.total, Some(6)); // 1 from seed + 5 extra

    // Page 2 (limit 3, offset 3)
    let page2 = service::get_all_comments(&pool, &topic, &post_slug, None, 3, 3, "new")
        .await
        .expect("page2");
    assert_eq!(page2.data.len(), 3);

    // Page 3 (offset 6 — should be empty)
    let page3 = service::get_all_comments(&pool, &topic, &post_slug, None, 3, 6, "new")
        .await
        .expect("page3");
    assert_eq!(page3.data.len(), 0);
}

#[tokio::test]
async fn comments_anon_token() {
    let pool = common::get_db_pool().await;
    let (topic, post_slug, _) = common::seed_minimal(&pool).await;

    // Without user_id
    let result = service::get_all_comments(&pool, &topic, &post_slug, None, 25, 0, "new")
        .await
        .expect("get_all_comments");
    assert!(result.data[0].anon_token.is_none());
    assert!(result.data[0].is_mine.is_none());

    // With viewer user_id
    let result = service::get_all_comments(&pool, &topic, &post_slug, Some(101), 25, 0, "new")
        .await
        .expect("get_all_comments with auth");
    assert!(result.data[0].anon_token.is_some());
    assert!(result.data[0].is_mine.unwrap()); // sender_id = 101
}

#[tokio::test]
async fn comments_content_too_long() {
    let pool = common::get_db_pool().await;
    let (topic, post_slug, _) = common::seed_minimal(&pool).await;

    let long_content = "x".repeat(50001);
    let err = service::create_comment(&pool, 200, &topic, &post_slug, &long_content)
        .await
        .expect_err("should reject content > 50k chars");
    assert_eq!(err.to_string(), "Content exceeds maximum allowed length");
}
