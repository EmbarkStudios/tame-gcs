#![cfg(feature = "signing")]

use reqwest::Client;
use tame_gcs::{signed_url, signing, BucketName, ObjectName};

struct Input {
    svc_account: signing::ServiceAccount,
    bucket: String,
    object: String,
}

impl Input {
    fn new() -> Self {
        use std::env;

        let ret = Self {
            svc_account: signing::ServiceAccount::load_json_file(
                env::var("TAME_GCS_TEST_SVC_ACCOUNT").expect("failed to get service account path"),
            )
            .expect("failed to load service account"),
            bucket: env::var("TAME_GCS_TEST_BUCKET").expect("failed to get test bucket"),
            object: env::var("TAME_GCS_TEST_OBJECT").expect("failed to get test object"),
        };

        BucketName::try_from(ret.bucket.as_str()).expect("invalid bucket name");
        ObjectName::try_from(ret.object.as_str()).expect("invalid object name");

        ret
    }

    fn bucket(&self) -> BucketName<'_> {
        BucketName::try_from(self.bucket.as_str()).unwrap()
    }

    fn object(&self) -> ObjectName<'_> {
        ObjectName::try_from(self.object.as_str()).unwrap()
    }
}

fn url_of_sadness(u: url::Url) -> reqwest::Url {
    reqwest::Url::parse(u.as_str()).unwrap()
}

#[ignore]
#[tokio::test]
async fn download_object() {
    let url_signer = signed_url::UrlSigner::with_ring();

    let input = Input::new();

    let signed = url_signer
        .generate(
            &input.svc_account,
            &(&input.bucket(), &input.object()),
            signed_url::SignedUrlOptional {
                //duration: std::time::Duration::from_secs(5),
                ..Default::default()
            },
        )
        .expect("signed url");

    let response = Client::new()
        .get(url_of_sadness(signed))
        .send()
        .await
        .expect("sent request")
        .error_for_status()
        .expect("successful request");

    response.bytes().await.expect("read body");
}

#[ignore]
#[tokio::test]
async fn gets_failure_responses_for_expired_urls() {
    let url_signer = signed_url::UrlSigner::with_ring();

    let input = Input::new();

    let signed = url_signer
        .generate(
            &input.svc_account,
            &(&input.bucket(), &input.object()),
            signed_url::SignedUrlOptional {
                duration: std::time::Duration::from_secs(1),
                ..Default::default()
            },
        )
        .expect("signed url");

    std::thread::sleep(std::time::Duration::from_millis(1500));

    let response = Client::new()
        .get(url_of_sadness(signed))
        .send()
        .await
        .expect("sent request");

    // We should get a failure response when trying to access a resource past its expiration
    assert_eq!(response.status(), 400);
}
