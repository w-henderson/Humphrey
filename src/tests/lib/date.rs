#[allow(unused_imports)]
use crate::http::date::DateTime;

#[test]
fn test_date_from_timestamp() {
    let input_0: i64 = 1628437415;
    let expected_output_0 = "Sun, 08 Aug 2021 15:43:35 GMT";
    let output_0 = DateTime::from(input_0).to_string();

    let input_1: i64 = 1094474096;
    let expected_output_1 = "Mon, 06 Sep 2004 12:34:56 GMT";
    let output_1 = DateTime::from(input_1).to_string();

    let input_2: i64 = 1584716400;
    let expected_output_2 = "Fri, 20 Mar 2020 15:00:00 GMT";
    let output_2 = DateTime::from(input_2).to_string();

    let input_3: i64 = 1582979696;
    let expected_output_3 = "Sat, 29 Feb 2020 12:34:56 GMT";
    let output_3 = DateTime::from(input_3).to_string();

    let input_4: i64 = -84337067;
    let expected_output_4 = "Sun, 30 Apr 1967 21:02:13 GMT";
    let output_4 = DateTime::from(input_4).to_string();

    let input_5: i64 = -28504100829;
    let expected_output_5 = "Fri, 28 Sep 1066 10:12:51 GMT";
    let output_5 = DateTime::from(input_5).to_string();

    assert_eq!(output_0, expected_output_0);
    assert_eq!(output_1, expected_output_1);
    assert_eq!(output_2, expected_output_2);
    assert_eq!(output_3, expected_output_3);
    assert_eq!(output_4, expected_output_4);
    assert_eq!(output_5, expected_output_5);
}
