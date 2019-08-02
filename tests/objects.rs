use std::convert::TryFrom;
use tame_gcs::{
    common::{Conditionals, StandardQueryParameters},
    objects::{self, DeleteObjectOptional, InsertObjectOptional, Metadata, Object},
    BucketName, ObjectId, ObjectName,
};

mod util;

#[test]
fn insert_vanilla() {
    let insert_req = Object::insert_simple(
        &(
            &BucketName::non_validated("bucket"),
            &ObjectName::non_validated("object/with/deep/path"),
        ),
        "great content",
        13,
        None,
    )
    .unwrap();

    let expected = http::Request::builder()
        .method(http::Method::POST)
        .uri("https://www.googleapis.com/upload/storage/v1/b/bucket/o?name=object/with/deep/path&uploadType=media&prettyPrint=false")
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
fn vanilla_get() {
    let get_req = Object::get(
        &ObjectId::new("bucket", "test/with/path_separators").unwrap(),
        None,
    )
    .unwrap();

    let expected = http::Request::builder()
        .method(http::Method::GET)
        .uri("https://www.googleapis.com/storage/v1/b/bucket/o/test%2Fwith%2Fpath_separators?alt=json&prettyPrint=false")
        .body(std::io::empty())
        .unwrap();

    util::requests_eq(&get_req, &expected);
}

#[test]
fn delete_vanilla() {
    let delete_req = Object::delete(
        &(
            &BucketName::non_validated("bucket"),
            &ObjectName::non_validated("object/with/deep/path"),
        ),
        None,
    )
    .unwrap();

    let expected = http::Request::builder()
        .method(http::Method::DELETE)
        .uri("https://www.googleapis.com/storage/v1/b/bucket/o/object%2Fwith%2Fdeep%2Fpath?prettyPrint=false")
        .body(std::io::empty())
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
        .body(std::io::empty())
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
        .body(std::io::empty())
        .unwrap();

    util::requests_eq(&delete_req, &expected);
}

#[test]
fn list_prefix_and_delimit() {
    let list_req = Object::list(
        &BucketName::non_validated("cache"),
        Some(objects::ListOptional {
            delimiter: Some("/"),
            prefix: Some("testing/"),
            ..Default::default()
        }),
    )
    .unwrap();

    let expected = http::Request::builder()
        .method(http::Method::GET)
        .uri("https://www.googleapis.com/storage/v1/b/cache/o?prettyPrint=false&delimiter=%2F&prefix=testing%2F")
        .body(std::io::empty())
        .unwrap();

    util::requests_eq(&list_req, &expected);
}

#[test]
fn parses_list_response() {
    let body = r#"{"kind":"storage#objects","prefixes":["testing/subdir/"],"items":[{"kind":"storage#object","id":"cache/testing/.gitignore/1563464155846959","selfLink":"https://www.googleapis.com/storage/v1/b/cache/o/testing%2F.gitignore","name":"testing/.gitignore","bucket":"cache","generation":"1563464155846959","metageneration":"1","contentType":"application/octet-stream","timeCreated":"2019-07-18T15:35:55.846Z","updated":"2019-07-18T15:35:55.846Z","storageClass":"REGIONAL","timeStorageClassUpdated":"2019-07-18T15:35:55.846Z","size":"30","md5Hash":"gVBKyp57x/mn4QvE+0fLvg==","mediaLink":"https://www.googleapis.com/download/storage/v1/b/cache/o/testing%2F.gitignore?generation=1563464155846959&alt=media","contentLanguage":"en","crc32c":"f+2iuw==","etag":"CK+yg+3lvuMCEAE="},{"kind":"storage#object","id":"cache/testing/test.zstd/1563439578444057","selfLink":"https://www.googleapis.com/storage/v1/b/cache/o/testing%2Ftest.zstd","name":"testing/test.zstd","bucket":"cache","generation":"1563439578444057","metageneration":"1","timeCreated":"2019-07-18T08:46:18.443Z","updated":"2019-07-18T08:46:18.443Z","storageClass":"REGIONAL","timeStorageClassUpdated":"2019-07-18T08:46:18.443Z","size":"688753933","md5Hash":"UQVzf70LIALAl6hdKnNnnA==","mediaLink":"https://www.googleapis.com/download/storage/v1/b/cache/o/testing%2Ftest.zstd?generation=1563439578444057&alt=media","crc32c":"OFE4Lg==","etag":"CJnizaWKvuMCEAE="}]}"#;

    let response = http::Response::new(body);
    let list_response = objects::ListResponse::try_from(response).expect("parsed list response");

    assert_eq!(2, list_response.objects.len());
    assert!(list_response.page_token.is_none());
}

#[test]
fn parses_empty_list_response() {
    let body = r#"{"kind":"storage#objects"}"#;

    let response = http::Response::new(body);
    let list_response = objects::ListResponse::try_from(response).expect("parsed list response");

    assert_eq!(0, list_response.objects.len());
    assert!(list_response.page_token.is_none());
}

const TEST_CONTENT: &str = include_str!("../CODE_OF_CONDUCT.md");

#[test]
fn insert_multipart_text() {
    let body = TEST_CONTENT;

    let metadata = Metadata {
        name: Some("good_name".to_owned()),
        content_type: Some("text/plain".to_owned()),
        metadata: Some(
            ["akey"]
                .iter()
                .map(|k| (String::from(*k), format!("{}value", k)))
                .collect(),
        ),
        ..Default::default()
    };

    let insert_req = Object::insert_multipart(
        &BucketName::non_validated("bucket"),
        std::io::Cursor::new(body),
        body.len() as u64,
        &metadata,
        None,
    )
    .unwrap();

    // Example request from https://cloud.google.com/storage/docs/json_api/v1/how-tos/multipart-upload
    // POST https://www.googleapis.com/upload/storage/v1/b/myBucket/o?uploadType=multipart HTTP/1.1
    // Authorization: Bearer [YOUR_AUTH_TOKEN]
    // Content-Type: multipart/related; boundary=foo_bar_baz
    // Content-Length: [NUMBER_OF_BYTES_IN_ENTIRE_REQUEST_BODY]

    // --foo_bar_baz
    // Content-Type: application/json; charset=UTF-8

    // {
    // "name": "myObject"
    // }

    // --foo_bar_baz
    // Content-Type: image/jpeg

    // [JPEG_DATA]
    // --foo_bar_baz--

    // We use `tame_gcs` as the boundary

    let expected_body = format!(
        "--{b}\ncontent-type: application/json; charset=utf-8\n\n{}\n--{b}\ncontent-type: text/plain\n\n{}\n--{b}--",
        serde_json::to_string(&metadata).unwrap(),
        body,
        b = "tame_gcs"
    );

    let expected = http::Request::builder()
        .method(http::Method::POST)
        .uri("https://www.googleapis.com/upload/storage/v1/b/bucket/o?uploadType=multipart&prettyPrint=false")
        .header(http::header::CONTENT_TYPE, "multipart/related; boundary=tame_gcs")
        .header(http::header::CONTENT_LENGTH, 3549)
        .body(std::io::Cursor::new(expected_body))
        .unwrap();

    util::requests_read_eq(insert_req, expected);
}

#[test]
fn multipart_read_paranoid() {
    // Ensure the Read implementation for Multipart works even with
    // a (hopefully) unrealistic case of copying 1 byte at a time
    let body = TEST_CONTENT;

    let metadata = Metadata {
        name: Some("a-really-descriptive-name".to_owned()),
        content_type: Some("text/plain".to_owned()),
        metadata: Some(
            ["key_one", "key_two", "should_sort_first"]
                .iter()
                .map(|k| (String::from(*k), format!("{}value", k)))
                .collect(),
        ),
        ..Default::default()
    };

    let mut mp =
        objects::Multipart::wrap(std::io::Cursor::new(body), body.len() as u64, &metadata).unwrap();

    let expected_body = format!(
        "--{b}\ncontent-type: application/json; charset=utf-8\n\n{}\n--{b}\ncontent-type: text/plain\n\n{}\n--{b}--",
        serde_json::to_string(&metadata).unwrap(),
        body,
        b = "tame_gcs"
    );

    use std::io::Read;
    let mut actual_body = Vec::with_capacity(expected_body.len());
    loop {
        let mut buf = [0; 1];
        if mp.read(&mut buf).unwrap() == 0 {
            break;
        }

        actual_body.push(buf[0]);
    }

    util::cmp_strings(&expected_body, &String::from_utf8_lossy(&actual_body));
}
