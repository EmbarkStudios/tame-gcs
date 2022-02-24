use http::Request;
use std::io::Read;

pub fn cmp_strings(expected: &str, actual: &str) {
    if expected != actual {
        let cs = difference::Changeset::new(expected, actual, "\n");
        panic!("{}", cs);
    }
}

pub fn requests_eq<AB: std::fmt::Debug, EB: std::fmt::Debug>(
    actual: &Request<AB>,
    expected: &Request<EB>,
) {
    let expected = format!("{:#?}", expected);
    let actual = format!("{:#?}", actual);

    cmp_strings(&expected, &actual);
}

#[allow(dead_code)]
pub fn requests_read_eq<AB: Read, EB: Read>(actual: Request<AB>, expected: Request<EB>) {
    let (ap, mut ab) = actual.into_parts();
    let (ep, mut eb) = expected.into_parts();

    let expected = format!("{:#?}", ep);
    let actual = format!("{:#?}", ap);

    cmp_strings(&expected, &actual);

    let mut act_bod = Vec::with_capacity(2 * 1024);
    ab.read_to_end(&mut act_bod).unwrap();

    let mut exp_bod = Vec::with_capacity(2 * 1024);
    eb.read_to_end(&mut exp_bod).unwrap();

    let act_body = String::from_utf8_lossy(&act_bod);
    let exp_body = String::from_utf8_lossy(&exp_bod);

    cmp_strings(&exp_body, &act_body);
}
