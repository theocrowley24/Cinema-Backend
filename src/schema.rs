table! {
    channels_tokens (id) {
        id -> Int4,
        token_id -> Int4,
        channel_user_id -> Int4,
        expires -> Timestamp,
        converted -> Bool,
    }
}

table! {
    comment_upvotes (id) {
        id -> Int4,
        user_id -> Int4,
        comment_id -> Int4,
        inactive -> Bool,
        date -> Timestamp,
        upvote_type -> Varchar,
    }
}

table! {
    comments (id) {
        id -> Int4,
        user_id -> Int4,
        text -> Varchar,
        inactive -> Bool,
        date -> Timestamp,
        video_id -> Int4,
    }
}

table! {
    tags (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    token_transactions (id) {
        id -> Int4,
        channel_user_id -> Int4,
        transaction_type -> Varchar,
        amount -> Int4,
        date -> Timestamp,
    }
}

table! {
    tokens (id) {
        id -> Int4,
        user_id -> Int4,
        used -> Bool,
        date_granted -> Timestamp,
        date_used -> Nullable<Timestamp>,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        email -> Varchar,
        password_reset_token -> Nullable<Varchar>,
        user_type -> Varchar,
        stripe_customer -> Varchar,
        subscribed -> Bool,
        stripe_account -> Nullable<Varchar>,
        channel_onboarded -> Bool,
        avatar_filename -> Nullable<Varchar>,
        cover_filename -> Nullable<Varchar>,
        subscriptions_enabled -> Bool,
        display_name -> Nullable<Varchar>,
        bio -> Nullable<Varchar>,
    }
}

table! {
    video_plays (id) {
        id -> Int4,
        user_id -> Int4,
        video_id -> Int4,
        date -> Timestamp,
    }
}

table! {
    video_upvotes (id) {
        id -> Int4,
        user_id -> Int4,
        video_id -> Int4,
        inactive -> Bool,
        date -> Timestamp,
        upvote_type -> Varchar,
    }
}

table! {
    videos (id) {
        id -> Int4,
        file_name -> Varchar,
        user_id -> Int4,
        title -> Varchar,
        description -> Nullable<Varchar>,
        upload_date -> Timestamp,
        status -> Nullable<Varchar>,
    }
}

table! {
    videos_tags (id) {
        id -> Int4,
        video_id -> Int4,
        tag_id -> Int4,
    }
}

joinable!(channels_tokens -> tokens (token_id));
joinable!(videos_tags -> tags (tag_id));
joinable!(videos -> users (user_id));
joinable!(comments -> users (user_id));
joinable!(comments -> videos (user_id));
joinable!(videos -> video_upvotes (id));
joinable!(comments -> comment_upvotes (id));
joinable!(video_plays -> videos (video_id));

allow_tables_to_appear_in_same_query!(
    channels_tokens,
    comment_upvotes,
    comments,
    tags,
    token_transactions,
    tokens,
    users,
    video_plays,
    video_upvotes,
    videos,
    videos_tags,
);
