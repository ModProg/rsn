use alloc::borrow::Cow;
use alloc::vec;
use core::fmt::Debug;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct StructOfEverything<'a> {
    str: Cow<'a, str>,
    bytes: serde_bytes::ByteBuf,
    char: char,
    u8: u8,
    u16: u16,
    u32: u32,
    u64: u64,
    u128: u128,
    usize: usize,
    i8: i8,
    i16: i16,
    i32: i32,
    i64: i64,
    i128: i128,
    isize: isize,
    bool: bool,
}

impl<'a> StructOfEverything<'a> {
    fn min() -> Self {
        Self {
            str: Cow::Borrowed("\0"),
            bytes: serde_bytes::ByteBuf::from(vec![0]),
            char: '\0',
            u8: 0,
            u16: 0,
            u32: 0,
            u64: 0,
            u128: 0,
            usize: 0,
            i8: i8::MIN,
            i16: i16::MIN,
            i32: i32::MIN,
            i64: i64::MIN,
            i128: i128::from(i64::MIN), /* To make deserialization strings consistent and compatible across feature flags */
            isize: isize::MIN,
            bool: false,
        }
    }

    fn max() -> Self {
        Self {
            str: Cow::Borrowed("hello \u{1_F980}"),
            bytes: serde_bytes::ByteBuf::from(b"hello, world".to_vec()),
            char: '\u{1_F980}',
            u8: u8::MAX,
            u16: u16::MAX,
            u32: u32::MAX,
            u64: u64::MAX,
            u128: u128::from(u64::MAX), /* To make deserialization strings consistent and compatible across feature flags */
            usize: usize::MAX,
            i8: i8::MAX,
            i16: i16::MAX,
            i32: i32::MAX,
            i64: i64::MAX,
            i128: i128::from(i64::MAX), /* To make deserialization strings consistent and compatible across feature flags */
            isize: isize::MAX,
            bool: true,
        }
    }
}

#[track_caller]
fn roundtrip<T: Debug + Serialize + for<'de> Deserialize<'de> + PartialEq>(value: &T, check: &str) {
    let rendered = crate::to_string(value);
    #[cfg(feature = "std")]
    {
        std::dbg!(&rendered);
    }
    assert_eq!(rendered, check);
    let restored: T = crate::from_str(&rendered).expect("deserialization failed");
    assert_eq!(&restored, value);
}

#[test]
fn struct_of_everything() {
    roundtrip(&StructOfEverything::default(), "StructOfEverything{str:\"\",bytes:b\"\",char:'\\0',u8:0,u16:0,u32:0,u64:0,u128:0,usize:0,i8:0,i16:0,i32:0,i64:0,i128:0,isize:0,bool:false}");
    roundtrip(&StructOfEverything::min(), "StructOfEverything{str:\"\\0\",bytes:b\"\\0\",char:'\\0',u8:0,u16:0,u32:0,u64:0,u128:0,usize:0,i8:-128,i16:-32768,i32:-2147483648,i64:-9223372036854775808,i128:-9223372036854775808,isize:-9223372036854775808,bool:false}");
    roundtrip(&StructOfEverything::max(), "StructOfEverything{str:\"hello 🦀\",bytes:b\"hello, world\",char:'🦀',u8:255,u16:65535,u32:4294967295,u64:18446744073709551615,u128:18446744073709551615,usize:18446744073709551615,i8:127,i16:32767,i32:2147483647,i64:9223372036854775807,i128:9223372036854775807,isize:9223372036854775807,bool:true}");
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(untagged)]
enum UntaggedEnum {
    Simple(SimpleStruct),
    NewtypeBool(NewtypeBool),
    Unit(UnitStruct),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct SimpleStruct {
    a: u64,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct NewtypeBool(bool);

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct UnitStruct;

#[test]
fn deserialize_any() {
    let untagged: UntaggedEnum = crate::from_str("()").unwrap();
    assert_eq!(untagged, UntaggedEnum::Unit(UnitStruct));
    let untagged: UntaggedEnum = crate::from_str("true").unwrap();
    assert_eq!(untagged, UntaggedEnum::NewtypeBool(NewtypeBool(true)));

    let untagged: UntaggedEnum = crate::from_str("{a:0}").unwrap();
    assert_eq!(untagged, UntaggedEnum::Simple(SimpleStruct { a: 0 }));
    // Serde doesn't support tagged in an untagged context, which makes sense
    // given what it's named. We can't pass the C to the visitor without causing
    // an error within deserialize_any() or causing it to think we're
    // deserializing only a string.
    let untagged: UntaggedEnum = crate::from_str("C{a:0}").unwrap();
    assert_eq!(untagged, UntaggedEnum::Simple(SimpleStruct { a: 0 }));

    // Some and None are special cases
    let untagged: Option<UntaggedEnum> = crate::from_str("None").unwrap();
    assert_eq!(untagged, None);
    let untagged: Option<UntaggedEnum> = crate::from_str("Some(())").unwrap();
    assert_eq!(untagged, Some(UntaggedEnum::Unit(UnitStruct)));
}
