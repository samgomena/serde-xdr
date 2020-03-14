use serde;

pub mod deserializer;
pub mod errors;
pub mod serializer;

pub use errors::{DecoderResult, EncoderError, EncoderResult};
use serde::{Deserialize, Serialize};
use std::io::Read;

pub use self::deserializer::Deserializer;
pub use self::serializer::Serializer;

pub fn to_bytes<T>(value: &T, buf: &mut Vec<u8>) -> EncoderResult<()>
where
    T: Serialize,
{
    let mut ser = Serializer::new(buf);
    value.serialize(&mut ser)?;
    Ok(())
}

pub fn from_reader<'a, T, R>(reader: R) -> DecoderResult<(T, usize)>
where
    T: Deserialize<'a>,
    R: Read + 'a,
{
    let mut de = Deserializer::new(reader);
    let value = Deserialize::deserialize(&mut de)?;
    Ok((value, de.get_bytes_consumed()))
}

pub fn from_bytes<'a, T>(v: &'a [u8]) -> DecoderResult<(T, usize)>
where
    T: Deserialize<'a>,
{
    from_reader(v)
}

#[macro_export]
macro_rules! xdr_enum {
    ($name:ident { $($variant:ident = $value:expr, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name<'a> {
            $($variant = $value,)*
        }

        impl<'a> ::serde::Serialize<'a> for $name<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::Serializer {
                serializer.serialize_i32(*self as i32) // All Enums are signed ints in XDR
            }
        }

        impl<'a> ::serde::Deserialize<'a> for $name<'a> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: ::serde::Deserializer<'a> {

                struct Visitor;

                impl<'a> ::serde::de::Visitor<'a> for Visitor {
                    type Value = $name<'a>;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("i32")
                    }

                    fn visit_i32<E>(self, value: i32) -> Result<$name<'a>, E> where E: ::serde::de::Error {
                        match value {
                            $( $value => Ok($name::$variant), )*
                            _ => Err(E::custom(
                                format!("unknown {} value: {}",
                                stringify!($name), value))),
                        }
                    }
                }
                deserializer.deserialize_i32(Visitor)
            }
        }
    }
}
