use easy_errors::define_errors;

define_errors!(
    PostError {
        UniqueViolation => {
            code: "23505",
            status: CONFLICT,
            message: "Resource already exists"
        },
        TopicNotFound => {
            code: "P0000",
            status: NOT_FOUND,
            message: "Topic not found"
        },
        PostNotFound => {
            code: "P0001",
            status: NOT_FOUND,
            message: "Post not found"
        },
        CommentNotFound => {
            code: "P0002",
            status: NOT_FOUND,
            message: "Comment not found"
        },
        ReplyNotFound => {
            code: "P0003",
            status: NOT_FOUND,
            message: "Reply not found"
        },
        InvalidTopicName => {
            code: "P0004",
            status: BAD_REQUEST,
            message: "Topic name may contain only letters, digits, and underscores"
        },
        InvalidVoteDirection => {
            code: "P0005",
            status: BAD_REQUEST,
            message: "Vote direction must be -1, 0, or 1"
        },
        ContentTooLong => {
            code: "P0007",
            status: BAD_REQUEST,
            message: "Content exceeds maximum allowed length"
        }
    }
);
