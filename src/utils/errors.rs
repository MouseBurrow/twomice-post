use burrow_db::define_errors;

define_errors!(
    PostError {
        UniqueViolation => "23505",
        TopicNotFound => "P0000",
        PostNotFound => "P0001",
        CommentNotFound => "P0002",
        ReplyNotFound => "P0003"

    }
);
