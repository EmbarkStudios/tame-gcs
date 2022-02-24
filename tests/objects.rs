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
            content_encoding: Some("identity"),
            ..Default::default()
        }),
    )
    .unwrap();

    let expected = http::Request::builder()
        .method(http::Method::POST)
        .uri("https://www.googleapis.com/upload/storage/v1/b/bucket/o?name=json&uploadType=media&prettyPrint=false&contentEncoding=identity")
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
        content_encoding: Some("gzip".to_owned()),
        content_disposition: Some("attachment; filename=\"good name.jpg\"".to_owned()),
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
        .header(http::header::CONTENT_LENGTH, 5758)
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
        content_encoding: Some("deflate".to_owned()),
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

#[cfg(feature = "async-multipart")]
#[test]
fn insert_multipart_async() {
    use futures_test::{io::AsyncReadTestExt, task::noop_context};
    use futures_util::{
        io::{AsyncRead, Cursor},
        pin_mut,
        task::Poll,
    };

    let body = TEST_CONTENT;

    let metadata = Metadata {
        name: Some("good_name".to_owned()),
        content_type: Some("text/plain".to_owned()),
        content_encoding: Some("gzip".to_owned()),
        content_disposition: Some("attachment; filename=\"good name.jpg\"".to_owned()),
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
        Cursor::new(body),
        body.len() as u64,
        &metadata,
        None,
    )
    .unwrap();

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
        .header(http::header::CONTENT_LENGTH, 5758)
        .body(std::io::Cursor::new(expected_body))
        .unwrap();

    let (ap, ab) = insert_req.into_parts();
    let (ep, mut eb) = expected.into_parts();

    let expected = format!("{:#?}", ep);
    let actual = format!("{:#?}", ap);

    util::cmp_strings(&expected, &actual);

    let mut act_bod = Vec::with_capacity(2 * 1024);
    {
        let reader = ab.interleave_pending();
        pin_mut!(reader);
        let mut cx = noop_context();
        let mut buff = [0; 20];
        loop {
            match reader.as_mut().poll_read(&mut cx, &mut buff) {
                Poll::Ready(size) => {
                    let size = size.unwrap();
                    if size == 0 {
                        break;
                    }

                    act_bod.extend_from_slice(&buff[..size]);
                }
                Poll::Pending => {}
            }
        }
    }

    use std::io::Read;
    let mut exp_bod = Vec::with_capacity(2 * 1024);
    eb.read_to_end(&mut exp_bod).unwrap();

    let act_body = String::from_utf8_lossy(&act_bod);
    let exp_body = String::from_utf8_lossy(&exp_bod);

    util::cmp_strings(&exp_body, &act_body);
}

#[cfg(feature = "async-multipart")]
#[test]
fn insert_multipart_stream_bytes() {
    use bytes::{BufMut, Bytes, BytesMut};

    let metadata = Metadata {
        name: Some("good_name".to_owned()),
        content_type: Some("text/plain".to_owned()),
        content_encoding: Some("gzip".to_owned()),
        content_disposition: Some("attachment; filename=\"good name.jpg\"".to_owned()),
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
        Bytes::from(TEST_CONTENT),
        TEST_CONTENT.len() as u64,
        &metadata,
        None,
    )
    .unwrap();

    let exp_body = format!(
        "--{b}\ncontent-type: application/json; charset=utf-8\n\n{}\n--{b}\ncontent-type: text/plain\n\n{}\n--{b}--",
        serde_json::to_string(&metadata).unwrap(),
        TEST_CONTENT,
        b = "tame_gcs"
    );

    let expected = http::Request::builder()
        .method(http::Method::POST)
        .uri("https://www.googleapis.com/upload/storage/v1/b/bucket/o?uploadType=multipart&prettyPrint=false")
        .header(http::header::CONTENT_TYPE, "multipart/related; boundary=tame_gcs")
        .header(http::header::CONTENT_LENGTH, 5758)
        .body(exp_body)
        .unwrap();

    let (exp_parts, exp_body) = expected.into_parts();
    let (act_parts, act_multipart) = insert_req.into_parts();

    util::cmp_strings(&format!("{:#?}", exp_parts), &format!("{:#?}", act_parts));

    let mut act_body = BytesMut::with_capacity(2 * 1024);
    for chunk in futures::executor::block_on_stream(act_multipart) {
        act_body.put(chunk);
    }
    let act_body = String::from_utf8_lossy(&act_body);

    util::cmp_strings(&exp_body, &act_body);
}

#[test]
fn patches() {
    let mut md = std::collections::BTreeMap::new();
    md.insert("yanked".to_owned(), "false".to_owned());

    let md = objects::Metadata {
        metadata: Some(md),
        ..Default::default()
    };

    let patch_req = Object::patch(&ObjectId::new("bucket", "object").unwrap(), &md, None).unwrap();

    let req_body = serde_json::to_vec(&md).unwrap();
    let expected_len = req_body.len();

    let expected = http::Request::builder()
        .method(http::Method::PATCH)
        .uri("https://storage.googleapis.com/storage/v1/b/bucket/o/object?prettyPrint=false")
        .header("content-type", "application/json")
        .header("content-length", expected_len)
        .body(std::io::Cursor::new(req_body))
        .unwrap();

    util::requests_read_eq(patch_req, expected);
}

#[test]
fn parses_patch_response() {
    let body = r#"{
        "kind": "storage#object",
        "id": "bucket/test-elf/1591708511706797",
        "selfLink": "https://www.googleapis.com/storage/v1/b/bucket/o/test-elf",
        "name": "test-elf",
        "bucket": "bucket",
        "generation": "1591708511706797",
        "metageneration": "2",
        "contentType": "application/x-elf",
        "timeCreated": "2020-06-09T13:15:11.706Z",
        "updated": "2020-06-09T13:20:53.073Z",
        "storageClass": "STANDARD",
        "timeStorageClassUpdated": "2020-06-09T13:15:11.706Z",
        "size": "11943404",
        "md5Hash": "oIyGCnAge5QkDf7UjVYwgQ==",
        "mediaLink": "https://content-storage.googleapis.com/download/storage/v1/b/bucket/o/test-elf?generation=1591708511706797&alt=media",
        "contentEncoding": "zstd",
        "contentDisposition": "attachment; filename=\"ark-client\"",
        "metadata": {
         "yanked": "false",
         "triple": "x86_64-unknown-linux-gnu",
         "size": "41468496",
         "author": "Ark CI",
         "branch": "master",
         "config": "Release",
         "version": "d8c83bd2b4cf808da298a2b2bc4ed3648581c5e0",
         "hash": "z125TKX8ryaEHpsGyUZ8CGzbGnA9xp1m4834tQZ5vwfxq",
         "version_semver": "0.1.9-pre",
         "timestamp": "2020-05-28T06:11:38+00:00"
        },
        "crc32c": "7ClPhg==",
        "etag": "CK29tKPo9OkCEAI="
       }
       "#;

    let response = http::Response::new(body);
    let patch_response =
        objects::PatchObjectResponse::try_from(response).expect("parsed patch response");

    assert_eq!(patch_response.metadata.metadata.unwrap().len(), 10);
    assert_eq!(
        patch_response.metadata.time_created.unwrap(),
        time::macros::datetime!(2020-06-09 13:15:11.706 UTC)
    );
    assert_eq!(
        patch_response.metadata.updated.unwrap(),
        time::macros::datetime!(2020-06-09 13:20:53.073 UTC)
    );
}

#[test]
fn rewrites_simple() {
    let rewrite_req = Object::rewrite(
        &ObjectId::new("source", "object").unwrap(),
        &ObjectId::new("target", "object/target.sh").unwrap(),
        None,
        None,
        None,
    )
    .unwrap();

    let expected = http::Request::builder()
        .method(http::Method::POST)
        .uri("https://storage.googleapis.com/storage/v1/b/source/o/object/rewriteTo/b/target/o/object%2Ftarget.sh?prettyPrint=false")
        .body(std::io::Cursor::new(Vec::new()))
        .unwrap();

    util::requests_read_eq(rewrite_req, expected);
}

#[test]
fn rewrites_token() {
    let rewrite_req = Object::rewrite(
        &ObjectId::new("source", "object/source.sh").unwrap(),
        &ObjectId::new("target", "object/target.sh").unwrap(),
        Some("tokeymctoken".to_owned()),
        None,
        None,
    )
    .unwrap();

    let expected = http::Request::builder()
        .method(http::Method::POST)
        .uri("https://storage.googleapis.com/storage/v1/b/source/o/object%2Fsource.sh/rewriteTo/b/target/o/object%2Ftarget.sh?rewriteToken=tokeymctoken&prettyPrint=false")
        .body(std::io::Cursor::new(Vec::new()))
        .unwrap();

    util::requests_read_eq(rewrite_req, expected);
}

#[test]
fn rewrites_metadata() {
    let mut md = std::collections::BTreeMap::new();
    md.insert("ohhi".to_owned(), "there".to_owned());
    let md = objects::Metadata {
        metadata: Some(md),
        ..Default::default()
    };

    let rewrite_req = Object::rewrite(
        &ObjectId::new("source", "object/source.sh").unwrap(),
        &ObjectId::new("target", "object/target.sh").unwrap(),
        None,
        Some(&md),
        Some(objects::RewriteObjectOptional {
            max_bytes_rewritten_per_call: Some(20),
            ..Default::default()
        }),
    )
    .unwrap();

    let req_body = serde_json::to_vec(&md).unwrap();
    let expected_len = req_body.len();

    let expected = http::Request::builder()
        .method(http::Method::POST)
        .uri("https://storage.googleapis.com/storage/v1/b/source/o/object%2Fsource.sh/rewriteTo/b/target/o/object%2Ftarget.sh?prettyPrint=false&maxBytesRewrittenPerCall=20")
        .header("content-type", "application/json")
        .header("content-length", expected_len)
        .body(std::io::Cursor::new(req_body))
        .unwrap();

    util::requests_read_eq(rewrite_req, expected);
}

#[test]
fn deserializes_partial_rewrite_response() {
    let body = r#"{
        "kind": "storage#rewriteResponse",
        "totalBytesRewritten": "435",
        "objectSize": "436",
        "done": false,
        "rewriteToken": "tokendata"
      }"#;

    let response = http::Response::new(body);
    let rewrite_response =
        objects::RewriteObjectResponse::try_from(response).expect("parsed rewrite response");

    assert_eq!(rewrite_response.total_bytes_rewritten, 435);
    assert!(!rewrite_response.done);
    assert_eq!(rewrite_response.rewrite_token.unwrap(), "tokendata");
}

#[test]
fn deserializes_complete_rewrite_response() {
    let body = r#"{
        "kind": "storage#rewriteResponse",
        "totalBytesRewritten": "435",
        "objectSize": "435",
        "done": true,
        "resource": {
          "kind": "storage#object",
          "id": "bucket/script.sh/1613655147314255",
          "selfLink": "https://www.googleapis.com/storage/v1/b/bucket/o/script.sh",
          "mediaLink": "https://content-storage.googleapis.com/download/storage/v1/b/bucket/o/script.sh?generation=1613655147314255&alt=media",
          "name": "script.sh",
          "bucket": "bucket",
          "generation": "1613655147314255",
          "metageneration": "1",
          "storageClass": "STANDARD",
          "size": "435",
          "md5Hash": "M8CAuwyX6GWwOnF5XxvqRw==",
          "crc32c": "3kHdqA==",
          "etag": "CM/44e7F8+4CEAE=",
          "timeCreated": "2021-02-18T13:32:27.315Z",
          "updated": "2021-02-18T13:32:27.315Z",
          "timeStorageClassUpdated": "2021-02-18T13:32:27.315Z",
          "metadata": {
            "ohhi": "true"
          }
        }
      }"#;

    let response = http::Response::new(body);
    let rewrite_response =
        objects::RewriteObjectResponse::try_from(response).expect("parsed rewrite response");

    assert_eq!(rewrite_response.total_bytes_rewritten, 435);
    assert!(rewrite_response.done);
    assert_eq!(
        rewrite_response.metadata.unwrap().name.unwrap(),
        "script.sh"
    );
}
