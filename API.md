# Post Service API Reference

All endpoints are served behind the gateway at `/api` prefix.  
Full URL from browser: `https://host/api/b/announcements/nib` → gateway strips `/api` → post service gets `/b/announcements/nib`.

---

## Auth headers

| Header | Required? | Source | Description |
|---|---|---|---|
| `X-User-Id` | varies | gateway (from session) | User's snowflake ID as `i64` |
| `X-Session-Token` | only for protected routes | gateway | Session token for auth validation |

Endpoints listed as "protected" require `X-User-Id` (returns 401 if missing).  
Endpoints listed as "public" accept `X-User-Id` optionally — responses include `is_mine` and `anon_token` when present.

---

## Error format

All errors return JSON:
```json
{ "error": "error_code", "message": "Human readable message" }
```

| Status | error_code | Meaning |
|---|---|---|
| 404 | `topic_not_found` | Board does not exist |
| 404 | `post_not_found` | Post does not exist |
| 404 | `comment_not_found` | Comment does not exist |
| 404 | `reply_not_found` | Reply does not exist |
| 404 | `tag_not_found` | Tag not found |
| 409 | `unique_violation` | Resource already exists (duplicate slug/hash) |
| 400 | `invalid_topic_name` | Board name has invalid characters |
| 400 | `invalid_vote_direction` | Vote direction not -1, 0, or 1 |
| 400 | `content_too_long` | Content exceeds max length |
| 422 | `invalid_tags` | Tag not in board's allowed list |
| 500 | `internal_error` | Unexpected server error |

---

## Boards

### `GET /b/active` — Active boards (public)

Query params: `limit` (optional, default 8, max 50)

Response `200`:
```json
[
  { "name": "general", "description": "General discussion", "post_count": 42 },
  { "name": "tech",    "description": "Technology",         "post_count": 17 }
]
```

---

### `GET /b` — All boards (public)

Response `200`:
```json
[
  { "name": "general",     "description": "General discussion", "created_at": "...", "deleted": false },
  { "name": "introductions", "description": "New members",  "created_at": "...", "deleted": false }
]
```

---

### `POST /b` — Create board (protected)

Body:
```json
{ "name": "board_name", "description": "What this board is for" }
```

Constraints: `name` allows only alphanumeric + underscores.

Response `204` No Content.

---

### `GET /b/:topic` — Get board info (public)

Response `200`:
```json
{ "name": "general", "description": "General discussion", "created_at": "...", "deleted": false }
```

---

### `GET /b/:topic/tags` — List allowed tags for board (public)

Response `200`:
```json
["rust", "typescript", "design", "bug"]
```

Returns empty array `[]` if board has no tag restrictions.

---

## Posts ("nibs")

### `GET /b/:topic/nib` — List posts in a board (public, auth-aware)

When `X-User-Id` is provided, each post includes `is_mine` and `anon_token`.

Query params: `_` (cache buster, ignored)

Response `200`:
```json
[
  {
    "title": "My first post",
    "slug": "3B7kA",
    "content": "Hello world!",
    "image_url": null,
    "created_at": "2026-06-11T12:00:00Z",
    "deleted": false,
    "vote_count": 5,
    "anon_token": null,
    "is_mine": true,
    "tags": ["rust", "typescript"],
    "reply_count": 3,
    "view_count": 120,
    "is_hot": true,
    "board_id": "announcements"
  }
]
```

| Field | Type | Description |
|---|---|---|
| `slug` | string | Base62-encoded snowflake ID, used as post identifier |
| `anon_token` | string or null | 16-char hex hash; present when viewer owns the post and board is anonymous |
| `is_mine` | bool or null | Whether the viewer owns this post (null if unauthenticated) |
| `is_hot` | bool | `true` if vote_count > 10 or view_count > 100 |
| `board_id` | string or null | Board name for frontend navigation links |
| `tags` | string[] | Post tags |
| `vote_count` | integer | Net vote score |
| `reply_count` | integer | Number of non-deleted comments + replies |
| `view_count` | integer | Total views |

---

### `POST /b/:topic/nib` — Create post (protected)

Body:
```json
{
  "title": "Post title",
  "content": "Post body content (max 50000 chars)",
  "image_url": "https://example.com/image.png",
  "tags": ["rust", "typescript"]
}
```

| Field | Required | Constraints |
|---|---|---|
| `title` | yes | Max 200 chars |
| `content` | yes | Max 50000 chars |
| `image_url` | no | Optional image URL |
| `tags` | no | Max 5 tags, each must be in board's allowed `topic_tags` list |

Response `204` No Content.

---

### `GET /b/:topic/nib/:post_id` — Get single post (public, auth-aware)

`post_id` can be a base62 slug or legacy slug.  
Increments `view_count` on each request.

Response `200`: Single `PostData` object (same shape as listing).

---

## Votes

### `POST /b/:topic/nib/:post_id/vote` — Vote on post (protected)

Body:
```json
{ "direction": 1 }
```

| direction | Meaning |
|---|---|
| `1` | Upvote |
| `-1` | Downvote |
| `0` | Remove vote |

Response `200`:
```json
{ "vote_count": 12 }
```

---

### `POST /b/:topic/nib/:post_id/sqk/:hash/vote` — Vote on comment (protected)

Body: same as post vote.

Response `200`:
```json
{ "vote_count": 5 }
```

---

## Comments ("squeaks")

### `GET /b/:topic/nib/:post_id/sqk` — List comments on a post (public, auth-aware)

When `X-User-Id` is provided, each comment includes `is_mine` and `anon_token`.

Response `200`:
```json
[
  {
    "hash": "aB3x9",
    "content": "Great post!",
    "created_at": "2026-06-11T12:30:00Z",
    "deleted": false,
    "vote_count": 3,
    "anon_token": "a1b2c3d4e5f6g7h8",
    "is_mine": true
  }
]
```

| Field | Type | Description |
|---|---|---|
| `hash` | string | 5-char base62 unique identifier |
| `anon_token` | string | Always present when authenticated. Deterministic per (viewer, sender, board, post) — lets viewers see which comments belong to the same author without revealing user IDs |

---

### `POST /b/:topic/nib/:post_id/sqk` — Create comment (protected)

Body:
```json
{ "content": "Comment text (max 50000 chars)" }
```

Response `204` No Content.

---

## Replies ("echoes")

### `GET /b/:topic/nib/:post_id/sqk/:hash/echoes` — List replies to a comment (public, auth-aware)

When `X-User-Id` is provided, each reply includes `is_mine` and `anon_token`.

Query params:

| Param | Type | Default | Max | Description |
|---|---|---|---|---|
| `limit` | integer | 25 | 100 | Number of top-level replies to return |
| `offset` | integer | 0 | — | Offset for pagination (top-level only) |

Response `200` — top-level replies are paginated; nested children are returned in full:
```json
{
  "data": [
    {
      "hash": "xYz99",
      "content": "I agree!",
      "created_at": "...",
      "deleted": false,
      "vote_count": 3,
      "anon_token": "b2c3d4e5f6g7h8i9",
      "is_mine": true,
      "children": [
        {
          "hash": "aBc12",
          "content": "Nested reply",
          "created_at": "...",
          "deleted": false,
          "vote_count": 1,
          "anon_token": "c3d4e5f6g7h8i9j0",
          "is_mine": false,
          "children": []
        }
      ]
    }
  ],
  "total": 42,
  "limit": 25,
  "offset": 0
}
```

---

### `POST /b/:topic/nib/:post_id/sqk/:hash/echoes` — Create reply (protected)

Body:
```json
{
  "content": "Reply text (max 50000 chars)",
  "reply_hash": null
}
```

| Field | Required | Description |
|---|---|---|
| `content` | yes | Reply body |
| `reply_hash` | no | Hash of a parent reply to nest under. If `null`, replies directly to the comment. Must belong to the same post+comment. |

Response `204` No Content.

---

### `POST /b/:topic/nib/:post_id/sqk/:comment_hash/echoes/:reply_hash/vote` — Vote on reply (protected)

Body:
```json
{ "direction": 1 }
```

| direction | Meaning |
|---|---|
| `1` | Upvote |
| `-1` | Downvote |
| `0` | Remove vote |

Response `200`:
```json
{ "vote_count": 5 }
```

---

## Feed

### `GET /feed` — Global feed (public)

Query params: `sort` (optional, default `"hot"`)

| sort | Order |
|---|---|
| `"hot"` (default) | By vote_count DESC, then created_at DESC |
| `"new"` | By created_at DESC |
| `"top"` | Same as "hot" |

Does NOT include auth-aware fields (`is_mine`, `anon_token`).

Response `200`: Array of `PostData` (up to 100 posts).

---

## User Posts

### `GET /users/me/nibs` — Current user's posts (protected)

Returns all posts created by the authenticated user.  
`is_mine` is always `true`, `anon_token` is always computed.

Response `200`: Array of `PostData`.

---

## Internal / Admin

### `GET /internal/stats/:user_id` — User content stats (protected)

Requires authentication (any valid user can query any user_id).

Response `200`:
```json
{
  "post_count": 5,
  "comment_count": 12,
  "upvote_count": 34
}
```

| Field | Description |
|---|---|
| `post_count` | Non-deleted posts created by user |
| `comment_count` | Non-deleted comments created by user |
| `upvote_count` | Net upvote score across all user's posts and comments |

---

## Type reference

### PostData (returned by most GET endpoints)

```json
{
  "title":        "string",
  "slug":         "string",         // base62 snowflake ID
  "content":      "string",
  "image_url":    "string | null",
  "created_at":   "ISO 8601 datetime",
  "deleted":      "bool",
  "vote_count":   "integer",
  "anon_token":   "string | null",  // 16 hex chars, only for own posts
  "is_mine":      "bool | null",    // null if unauthenticated
  "tags":         "string[]",
  "reply_count":  "integer",
  "view_count":   "integer",
  "is_hot":       "bool",
  "board_id":     "string | null"   // board name for nav links
}
```

### CommentData

```json
{
  "hash":        "string",         // 5-char base62 ID
  "content":     "string",
  "created_at":  "ISO 8601 datetime",
  "deleted":     "bool",
  "vote_count":  "integer",
  "anon_token":  "string | null",  // 16 hex chars, always set when auth'd
  "is_mine":     "bool | null"
}
```

### ReplyData

```json
{
  "hash":        "string",         // 5-char base62 ID
  "content":     "string",
  "created_at":  "ISO 8601 datetime",
  "deleted":     "bool",
  "vote_count":  "integer",
  "anon_token":  "string | null",  // 16 hex chars, always set when auth'd
  "is_mine":     "bool | null",
  "children":    "ReplyData[]"     // nested replies, empty array if none
}
```

### BoardData

```json
{
  "name":        "string",
  "description": "string",
  "created_at":  "ISO 8601 datetime",
  "deleted":     "bool"
}
```

### BoardSummary

```json
{
  "name":        "string",
  "description": "string",
  "post_count":  "integer"
}
```

---

## Content limits

| Field | Max |
|---|---|
| `title` | 200 chars |
| `content` (post/comment/reply) | 50000 chars |
| `tags` per post | 5 tags |

## URL parameter naming

| URL segment | Meaning |
|---|---|
| `:topic` | Board name (e.g. `announcements`, `general`, `tech`) |
| `:post_id` | Post slug (base62 snowflake ID, e.g. `3B7kA`) |
| `:hash` | Comment hash (5-char base62, e.g. `aB3x9`) |
| `:comment_hash` | Comment hash (used in reply vote route) |
| `:reply_hash` | Reply hash (5-char base62, e.g. `xYz99`) |
