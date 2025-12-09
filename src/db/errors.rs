use burrow_db::define_errors;

define_errors!(
    PostError {
        UserNotFound => "P0000",
        PostNotFound => "P0001",
        CommentNotFound => "P0002"
    }
);
