# TODO — Post Service API Fixes

## 1. `/users/me/nibs` — Include `board_id` in response

### File: `services/post/src/service.rs`, line 630

**Current:**
```sql
NULL::TEXT as board_id
```

**Required:**
```sql
t.name as board_id
```

The query already joins `topics t ON t.id = p.topic_id`, so `t.name` is available.
This field is already present on the `PostData` struct (line 102) as `pub board_id: Option<String>`,
but the current SQL hardcodes `NULL`.

### Why:
The frontend `PostCard` component constructs navigation URLs as `/b/{board_id}/post/{slug}`.
Without `board_id`, clicking a post on the profile page is a no-op.

---

## 2. `anon_token` for own nibs

**File:** `services/post/src/service.rs`, line 624

**Current:**
```sql
NULL::TEXT as anon_token,
```

Users viewing their own nibs on their profile page see no anonymous badge because `anon_token`
is hardcoded to `NULL` in this query. The frontend checks `post.anon_token &&` before rendering
the `AnonBadge` component.

**Required:** Compute the anon token using the same `anon_id()` function used by other nib
queries (hash of `user_id + board_id + post_id` with a secret salt). This way the user can
see their own anon badge on their profile and verify what identity others will see.

---

## 3. `is_hot` should be computed

**File:** `services/post/src/service.rs`, line 629

**Current:**
```sql
false as is_hot,
```

The "hot" badge is never shown on user's own nibs list. Should use the same threshold logic
as other nib listings.
