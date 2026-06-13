mod common;

use post::service;

#[tokio::test]
async fn create_and_get_post() {
    let pool = common::get_db_pool().await;
    let (topic, _post_slug, _comment_hash) = common::seed_minimal(&pool).await;

    let slug = service::create_post(&pool, 200, &topic, "My Post", "Post content", &None, &None)
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

    service::create_reply(
        &pool,
        102,
        &topic,
        &post_slug,
        &comment_hash,
        "A reply",
        None,
    )
    .await
    .expect("create_reply should succeed");

    let post = service::get_post(&pool, &post_slug, None)
        .await
        .expect("get_post should succeed");
    assert_eq!(post.reply_count, 2); // 1 comment + 1 reply
}

#[tokio::test]
async fn feed_returns_hot_posts() {
    let pool = common::get_db_pool().await;
    common::seed_minimal(&pool).await;

    let posts = service::get_feed_posts(&pool, "hot", &config::app_envs::AppEnvs::DEV)
        .await
        .expect("get_feed_posts should succeed");
    assert!(!posts.is_empty());
    assert!(posts.iter().any(|p| p.title == "Test Post"));
}

#[tokio::test]
async fn feed_sort_new() {
    let pool = common::get_db_pool().await;
    common::seed_minimal(&pool).await;

    let posts = service::get_feed_posts(&pool, "new", &config::app_envs::AppEnvs::DEV)
        .await
        .expect("get_feed_posts(new) should succeed");
    assert!(!posts.is_empty());
}

#[tokio::test]
async fn feed_sort_top() {
    let pool = common::get_db_pool().await;
    common::seed_minimal(&pool).await;

    let posts = service::get_feed_posts(&pool, "top", &config::app_envs::AppEnvs::DEV)
        .await
        .expect("get_feed_posts(top) should succeed");
    assert!(!posts.is_empty());
}

#[tokio::test]
async fn board_create_and_get() {
    let pool = common::get_db_pool().await;
    common::seed_minimal(&pool).await;

    // get_board
    let board = service::get_board(&pool, "test-board")
        .await
        .expect("get_board should succeed");
    assert_eq!(board.name, "test-board");
}

#[tokio::test]
async fn board_list() {
    let pool = common::get_db_pool().await;
    common::seed_minimal(&pool).await;

    let boards = service::get_all_boards(&pool)
        .await
        .expect("get_all_boards should succeed");
    assert!(boards.iter().any(|b| b.name == "test-board"));
}

#[tokio::test]
async fn board_active() {
    let pool = common::get_db_pool().await;
    common::seed_minimal(&pool).await;

    let active = service::get_active_boards(&pool, 10)
        .await
        .expect("get_active_boards should succeed");
    assert!(active.iter().any(|b| b.name == "test-board"));
    assert!(active.iter().all(|b| b.post_count > 0));
}

#[tokio::test]
async fn user_posts_returns_correct_user() {
    let pool = common::get_db_pool().await;
    common::seed_minimal(&pool).await;

    // seed_minimal creates a post with creator_id = 100
    let posts = service::get_user_posts(&pool, 100)
        .await
        .expect("get_user_posts should succeed");
    assert!(!posts.is_empty());
    assert!(posts.iter().all(|p| p.title == "Test Post"));
}

#[tokio::test]
async fn user_posts_empty_for_unknown_user() {
    let pool = common::get_db_pool().await;
    common::seed_minimal(&pool).await;

    let posts = service::get_user_posts(&pool, 99999)
        .await
        .expect("get_user_posts should succeed");
    assert!(posts.is_empty());
}

#[tokio::test]
async fn user_content_stats() {
    let pool = common::get_db_pool().await;
    common::seed_minimal(&pool).await;

    let stats = service::get_user_content_stats(&pool, 100)
        .await
        .expect("get_user_content_stats should succeed");
    assert_eq!(stats.post_count, 1);
    assert_eq!(stats.comment_count, 0);
    assert_eq!(stats.upvote_count, 0);
}
