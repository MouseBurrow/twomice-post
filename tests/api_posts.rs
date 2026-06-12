mod common;

use post::service;

#[tokio::test]
async fn create_and_get_post() {
    let pool = common::get_db_pool().await;
    let (topic, _post_slug, _comment_hash) = common::seed_minimal(&pool).await;

    let slug = service::create_post(
        &pool, 200, &topic, "My Post", "Post content",
        &None, &None,
    )
    .await
    .expect("create_post should succeed");

    let post = service::get_post(&pool, &slug, None)
        .await
        .expect("get_post should succeed");
    assert_eq!(post.title, "My Post");
    assert_eq!(post.content, "Post content");
    assert_eq!(post.reply_count, 0); // no comments on this post yet

    let resolved = service::resolve_post_id(&pool, &topic, &slug)
        .await
        .expect("resolve_post_id should succeed");
    assert!(resolved > 0);
}

#[tokio::test]
async fn list_posts_in_board() {
    let pool = common::get_db_pool().await;
    let (topic, _slug, _) = common::seed_minimal(&pool).await;

    let posts = service::get_all_posts(&pool, &topic, None)
        .await
        .expect("get_all_posts should succeed");

    let titles: Vec<&str> = posts.iter().map(|p| p.title.as_str()).collect();
    assert!(titles.contains(&"Test Post"));
}

#[tokio::test]
async fn post_reply_count_includes_replies() {
    let pool = common::get_db_pool().await;
    let (topic, post_slug, comment_hash) = common::seed_minimal(&pool).await;

    service::create_reply(&pool, 102, &topic, &post_slug, &comment_hash, "A reply", None)
        .await
        .expect("create_reply should succeed");

    let post = service::get_post(&pool, &post_slug, None)
        .await
        .expect("get_post should succeed");
    assert_eq!(post.reply_count, 2); // 1 comment + 1 reply
}
