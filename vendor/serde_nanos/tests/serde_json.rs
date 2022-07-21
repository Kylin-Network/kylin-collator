use serde;
use serde_json;

#[cfg(feature = "chrono")]
mod std_time {
    use serde_derive::{Deserialize, Serialize};
    use std::time::Duration;

    #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct Payload {
        #[serde(with = "serde_nanos")]
        duration: Duration,
    }

    #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct Optional {
        #[serde(default, with = "serde_nanos")]
        duration: Option<Duration>,
    }

    #[test]
    fn zero_nanosecond_duration_from_str() {
        let actual = serde_json::from_str::<Payload>(r#"{ "duration": 0 }"#).unwrap();
        let expected = Payload {
            duration: Duration::from_millis(0),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn one_second_duration_from_str() {
        let actual = serde_json::from_str::<Payload>(r#"{ "duration": 1000000000  }"#).unwrap();
        let expected = Payload {
            duration: Duration::from_secs(1),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn default_to_none_from_empty_str() {
        let actual = serde_json::from_str::<Optional>(r#"{ }"#).unwrap();

        let expected = Optional { duration: None };

        assert_eq!(actual, expected);
    }

    #[test]
    fn zero_nanosecond_optional_from_str() {
        let actual = serde_json::from_str::<Optional>(r#"{ "duration": 0}"#).unwrap();

        let expected = Optional {
            duration: Some(Duration::from_millis(0)),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn one_second_optional_from_str() {
        let actual = serde_json::from_str::<Optional>(r#"{ "duration": 1000000000}"#).unwrap();

        let expected = Optional {
            duration: Some(Duration::from_secs(1)),
        };

        assert_eq!(actual, expected);
    }
}

#[cfg(feature = "chrono")]
mod chrono {
    use chrono::Duration;
    use serde_derive::{Deserialize, Serialize};

    #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct Payload {
        #[serde(with = "serde_nanos")]
        duration: Duration,
    }

    #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct Optional {
        #[serde(default, with = "serde_nanos")]
        duration: Option<Duration>,
    }

    #[test]
    fn zero_nanosecond_duration_from_str() {
        let actual = serde_json::from_str::<Payload>(r#"{ "duration": 0 }"#).unwrap();
        let expected = Payload {
            duration: Duration::nanoseconds(0),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn one_second_duration_from_str() {
        let actual = serde_json::from_str::<Payload>(r#"{ "duration": 1000000000  }"#).unwrap();
        let expected = Payload {
            duration: Duration::seconds(1),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn default_to_none_from_empty_str() {
        let actual = serde_json::from_str::<Optional>(r#"{ }"#).unwrap();

        let expected = Optional { duration: None };

        assert_eq!(actual, expected);
    }

    #[test]
    fn zero_nanosecond_optional_from_str() {
        let actual = serde_json::from_str::<Optional>(r#"{ "duration": 0}"#).unwrap();

        let expected = Optional {
            duration: Some(Duration::seconds(0)),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn one_second_optional_from_str() {
        let actual = serde_json::from_str::<Optional>(r#"{ "duration": 1000000000}"#).unwrap();

        let expected = Optional {
            duration: Some(Duration::seconds(1)),
        };

        assert_eq!(actual, expected);
    }
}
