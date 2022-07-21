use rustix::ffi::{ZStr, ZString};
use rustix::io;
use rustix::path::Arg;
#[cfg(feature = "itoa")]
use rustix::path::DecInt;
use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::path::{Component, Components, Iter, Path, PathBuf};

#[test]
fn test_arg() {
    use rustix::zstr;
    use std::borrow::Borrow;

    let t: &str = "hello";
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.into_z_str().unwrap()));

    let t: String = "hello".to_owned();
    assert_eq!("hello", Arg::as_str(&t).unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(
        zstr!("hello"),
        Borrow::borrow(&t.clone().into_z_str().unwrap())
    );

    let t: &OsStr = OsStr::new("hello");
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.into_z_str().unwrap()));

    let t: OsString = OsString::from("hello".to_owned());
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(
        zstr!("hello"),
        Borrow::borrow(&t.clone().into_z_str().unwrap())
    );

    let t: &Path = Path::new("hello");
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.into_z_str().unwrap()));

    let t: PathBuf = PathBuf::from("hello".to_owned());
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(
        zstr!("hello"),
        Borrow::borrow(&t.clone().into_z_str().unwrap())
    );

    let t: &ZStr = zstr!("hello");
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.into_z_str().unwrap()));

    let t: ZString = zstr!("hello").to_owned();
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(
        zstr!("hello"),
        Borrow::borrow(&Arg::as_cow_z_str(&t).unwrap())
    );
    assert_eq!(
        zstr!("hello"),
        Borrow::borrow(&t.clone().into_z_str().unwrap())
    );

    let t: Components = Path::new("hello").components();
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(
        zstr!("hello"),
        Borrow::borrow(&t.clone().into_z_str().unwrap())
    );

    let t: Component = Path::new("hello").components().next().unwrap();
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.into_z_str().unwrap()));

    let t: Iter = Path::new("hello").iter();
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(
        zstr!("hello"),
        Borrow::borrow(&t.clone().into_z_str().unwrap())
    );

    let t: Cow<'_, str> = Cow::Borrowed("hello");
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(
        zstr!("hello"),
        Borrow::borrow(&t.clone().into_z_str().unwrap())
    );

    let t: Cow<'_, str> = Cow::Owned("hello".to_owned());
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(
        zstr!("hello"),
        Borrow::borrow(&t.clone().into_z_str().unwrap())
    );

    let t: Cow<'_, OsStr> = Cow::Borrowed(OsStr::new("hello"));
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(
        zstr!("hello"),
        Borrow::borrow(&t.clone().into_z_str().unwrap())
    );

    let t: Cow<'_, OsStr> = Cow::Owned(OsString::from("hello".to_owned()));
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(
        zstr!("hello"),
        Borrow::borrow(&t.clone().into_z_str().unwrap())
    );

    let t: Cow<'_, ZStr> = Cow::Borrowed(zstr!("hello"));
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(
        zstr!("hello"),
        Borrow::borrow(&t.clone().into_z_str().unwrap())
    );

    let t: Cow<'_, ZStr> = Cow::Owned(zstr!("hello").to_owned());
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(
        zstr!("hello"),
        Borrow::borrow(&t.clone().into_z_str().unwrap())
    );

    let t: &[u8] = b"hello";
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.into_z_str().unwrap()));

    let t: Vec<u8> = b"hello".to_vec();
    assert_eq!("hello", t.as_str().unwrap());
    assert_eq!("hello".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(zstr!("hello"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
    assert_eq!(
        zstr!("hello"),
        Borrow::borrow(&t.clone().into_z_str().unwrap())
    );

    #[cfg(feature = "itoa")]
    {
        let t: DecInt = DecInt::new(43110);
        assert_eq!("43110", t.as_str());
        assert_eq!("43110".to_owned(), Arg::to_string_lossy(&t));
        assert_eq!(zstr!("43110"), Borrow::borrow(&t.as_cow_z_str().unwrap()));
        assert_eq!(zstr!("43110"), t.as_c_str());
        assert_eq!(
            zstr!("43110"),
            Borrow::borrow(&t.clone().into_z_str().unwrap())
        );
    }
}

#[test]
fn test_invalid() {
    use std::borrow::Borrow;

    let t: &[u8] = b"hello\xc0world";
    assert_eq!(t.as_str().unwrap_err(), io::Error::INVAL);
    assert_eq!("hello\u{fffd}world".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(
        b"hello\xc0world\0",
        Borrow::<ZStr>::borrow(&t.as_cow_z_str().unwrap()).to_bytes_with_nul()
    );
    assert_eq!(
        b"hello\xc0world\0",
        Borrow::<ZStr>::borrow(&t.clone().into_z_str().unwrap()).to_bytes_with_nul()
    );
}

#[test]
fn test_embedded_nul() {
    let t: &[u8] = b"hello\0world";
    assert_eq!("hello\0world", t.as_str().unwrap());
    assert_eq!("hello\0world".to_owned(), Arg::to_string_lossy(&t));
    assert_eq!(t.as_cow_z_str().unwrap_err(), io::Error::INVAL);
    assert_eq!(t.clone().into_z_str().unwrap_err(), io::Error::INVAL);
}
