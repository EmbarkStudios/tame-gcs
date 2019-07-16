use tame_gcs::{
    common::{Conditionals, StandardQueryParameters},
    objects::{DeleteObjectOptional, InsertObjectOptional, Object},
    BucketName, ObjectId, ObjectName,
};
mod util;

#[test]
fn insert_vanilla() {
    let insert_req = Object::insert_simple(
        &(
            &BucketName::non_validated("bucket"),
            &ObjectName::non_validated("object"),
        ),
        "great content",
        13,
        None,
    )
    .unwrap();

    let expected = http::Request::builder()
        .method(http::Method::POST)
        .uri("https://www.googleapis.com/upload/storage/v1/b/bucket/o?name=object&uploadType=media&prettyPrint=false")
        .header(http::header::CONTENT_TYPE, "application/octet-stream")
        .header(http::header::CONTENT_LENGTH, 13)
        .body("great content")
        .unwrap();

    util::requests_eq(&insert_req, &expected);
}

#[test]
fn insert_json_content() {
    let insert_req = Object::insert_simple(
        &ObjectId::new("bucket", "json").unwrap(),
        r#"{"data":23}"#,
        11,
        Some(InsertObjectOptional {
            content_type: Some("application/json"),
            ..Default::default()
        }),
    )
    .unwrap();

    let expected = http::Request::builder()
        .method(http::Method::POST)
        .uri("https://www.googleapis.com/upload/storage/v1/b/bucket/o?name=json&uploadType=media&prettyPrint=false")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::CONTENT_LENGTH, 11)
        .body(r#"{"data":23}"#)
        .unwrap();

    util::requests_eq(&insert_req, &expected);
}

#[test]
fn delete_vanilla() {
    let delete_req = Object::delete(
        &(
            &BucketName::non_validated("bucket"),
            &ObjectName::non_validated("object"),
        ),
        None,
    )
    .unwrap();

    let expected = http::Request::builder()
        .method(http::Method::DELETE)
        .uri("https://www.googleapis.com/storage/v1/b/bucket/o/object?prettyPrint=false")
        .body(())
        .unwrap();

    util::requests_eq(&delete_req, &expected);
}

#[test]
fn delete_some_optional() {
    let delete_req = Object::delete(
        &ObjectId::new("bucket", "object").unwrap(),
        Some(DeleteObjectOptional {
            generation: Some(20),
            conditionals: Conditionals {
                if_metageneration_not_match: Some(999),
                ..Default::default()
            },
            ..Default::default()
        }),
    )
    .unwrap();

    let expected = http::Request::builder()
        .method(http::Method::DELETE)
        .uri("https://www.googleapis.com/storage/v1/b/bucket/o/object?prettyPrint=false&generation=20&ifMetagenerationNotMatch=999")
        .body(())
        .unwrap();

    util::requests_eq(&delete_req, &expected);
}

#[test]
fn delete_all_optional() {
    let delete_req = Object::delete(
        &ObjectId::new("bucket", "object").unwrap(),
        Some(DeleteObjectOptional {
            standard_params: StandardQueryParameters {
                fields: Some("field1"),
                pretty_print: true,
                quota_user: Some("some-user"),
                user_ip: Some("some-user-ip"),
            },
            generation: Some(1),
            conditionals: Conditionals {
                if_generation_match: Some(2),
                if_generation_not_match: Some(3),
                if_metageneration_match: Some(4),
                if_metageneration_not_match: Some(5),
            },
            user_project: Some("some-user-project"),
        }),
    )
    .unwrap();

    let expected = http::Request::builder()
        .method(http::Method::DELETE)
        .uri("https://www.googleapis.com/storage/v1/b/bucket/o/object?fields=field1&quotaUser=some-user&userIp=some-user-ip&generation=1&ifGenerationMatch=2&ifGenerationNotMatch=3&ifMetagenerationMatch=4&ifMetagenerationNotMatch=5&userProject=some-user-project")
        .body(())
        .unwrap();

    util::requests_eq(&delete_req, &expected);
}
