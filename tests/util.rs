use http::Request;

pub fn requests_eq<T: std::fmt::Debug>(actual: &Request<T>, expected: &Request<T>) {
    let expected = format!("{:#?}", expected);
    let actual = format!("{:#?}", actual);

    if expected != actual {
        let cs = difference::Changeset::new(&expected, &actual, "\n");
        assert!(false, "{}", cs);
    }
}
