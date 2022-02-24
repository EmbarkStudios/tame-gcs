use tame_gcs::{
    objects::{Object, ResumableSession},
    BucketName, ObjectName,
};

mod util;

#[test]
fn resumable_init() {
    let insert_req = Object::resumable_insert_init(
        &(
            &BucketName::non_validated("bucket"),
            &ObjectName::non_validated("object/with/deep/path"),
        ),
        Some("application/json"),
    )
    .unwrap();

    let expected = http::Request::builder()
        .method(http::Method::POST)
        .uri("https://www.googleapis.com/upload/storage/v1/b/bucket/o?uploadType=resumable&name=object/with/deep/path")
        .header(http::header::CONTENT_LENGTH, 0)
        .header(http::header::HeaderName::from_static("x-upload-content-type"),
        http::header::HeaderValue::from_str("application/json").unwrap())
        .body(())
        .unwrap();

    util::requests_eq(&insert_req, &expected);
}

#[test]
fn resumable_cancel() {
    let session = ResumableSession("https://killedbygoogle.com/".parse().unwrap());

    let cancel_req = Object::resumable_cancel(session.clone()).unwrap();

    let expected = http::Request::builder()
        .method(http::Method::DELETE)
        .uri(session)
        .header(http::header::CONTENT_LENGTH, 0i32)
        .body(())
        .unwrap();

    util::requests_eq(&cancel_req, &expected);
}

#[test]
fn resumable_append() {
    let session = ResumableSession("https://killedbygoogle.com/".parse().unwrap());
    let content = r#"{"data":23}"#;

    let append_req = Object::resumable_append(session.clone(), content, 11).unwrap();

    let expected = http::Request::builder()
        .method(http::Method::PUT)
        .uri(session)
        .header(http::header::CONTENT_LENGTH, 11i32)
        .body(content)
        .unwrap();

    util::requests_eq(&append_req, &expected);
}
