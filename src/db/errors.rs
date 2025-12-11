use burrow_db::define_errors;

define_errors!(
    PostError {
        UniqueViolation => "23505",
        TopicNotFound => "P0000",
        PostNotFound => "P0001"
    }
);
