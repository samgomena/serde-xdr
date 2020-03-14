use crate::errors::{DecoderResult, EncoderError};

use byteorder::{BigEndian, ReadBytesExt};
use serde::de::{self, Deserialize, IntoDeserializer, Visitor};
use std::io::{self, Read};

macro_rules! not_implemented {
    ($($name:ident($($arg:ident: $ty:ty,)*);)*) => {
        $(fn $name<V: Visitor<'de>>(self, $($arg: $ty,)* _visitor: V) -> DecoderResult<V::Value> {
            Err(EncoderError::Unknown(format!("XDR deserialize not implemented for {}", stringify!($name))))
        })*
    }
}

// impl_num!(u16, deserialize_u16, visit_u16, read_u16, 2);
macro_rules! impl_num {
    ($ty:ty, $deserialize_method:ident, $visitor_method:ident, $read_method:ident, $byte_size:expr) => {
        fn $deserialize_method<V>(self, visitor: V) -> DecoderResult<V::Value>
            where V: Visitor<'de>, {
                let res = visitor.$visitor_method(self.$read_method::<BigEndian>()?);
                self.bytes_consumed += $byte_size;
                res
        }
    }
}

#[derive(Debug)]
pub struct Deserializer<R>
where
    R: Read,
{
    reader: R,
    bytes_consumed: usize,
}

impl<R> Deserializer<R>
where
    R: Read,
{
    pub fn new(reader: R) -> Deserializer<R> {
        Deserializer {
            reader,
            bytes_consumed: 0,
        }
    }

    pub fn get_bytes_consumed(&self) -> usize {
        self.bytes_consumed
    }
}

#[derive(Debug)]
enum XdrEnumType {
    Enum,
    Union,
}

impl<'de, 'a, R> de::Deserializer<'de> for &'a mut Deserializer<R>
where
    R: Read,
{
    type Error = EncoderError;

    // Implementing all the numbers that use the simple read_TYPE syntax
    impl_num!(u16, deserialize_u16, visit_u16, read_u16, 2);
    impl_num!(u32, deserialize_u32, visit_u32, read_u32, 4);
    impl_num!(u64, deserialize_u64, visit_u64, read_u64, 8);

    impl_num!(i16, deserialize_i16, visit_i16, read_i16, 2);
    impl_num!(i32, deserialize_i32, visit_i32, read_i32, 4);
    impl_num!(i64, deserialize_i64, visit_i64, read_i64, 8);

    impl_num!(f32, deserialize_f32, visit_f32, read_f32, 4);
    impl_num!(f64, deserialize_f64, visit_f64, read_f64, 8);

    not_implemented!(
        deserialize_char();
        deserialize_str();
        deserialize_unit();
        deserialize_option();
        deserialize_bytes();
        deserialize_map();
        deserialize_unit_struct(_name: &'static str,);
        deserialize_tuple_struct(_name: &'static str, _len: usize,);
        deserialize_tuple(_len: usize,);
        deserialize_ignored_any();
    );

    // See: deserialize_identifier
    // Docs: https://docs.serde.rs/serde/trait.Deserializer.html#tymethod.deserialize_identifier
    fn deserialize_identifier<V>(self, visitor: V) -> DecoderResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> DecoderResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let count: u32 = self.read_u32::<BigEndian>()?;
        let extra_bytes = 4 - count % 4;
        let mut accum = String::new();
        for _ in 0..count {
            accum.push(self.read_u8()? as char);
        }
        self.bytes_consumed += (extra_bytes + count + 4) as usize;
        visitor.visit_string(accum)
    }

    fn deserialize_enum<V>(
        self,
        name: &str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> DecoderResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if name == "__UNION_SYMBOL__" {
            visitor.visit_enum(VariantVisitor::new(self, XdrEnumType::Union, variants))
        } else {
            visitor.visit_enum(VariantVisitor::new(self, XdrEnumType::Enum, variants))
        }
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, mut _visitor: V) -> DecoderResult<V::Value> {
        Err(EncoderError::Unknown(String::from("not done implementing")))
    }

    fn deserialize_any<V: Visitor<'de>>(self, mut _visitor: V) -> DecoderResult<V::Value> {
        Err(EncoderError::Unknown(String::from(
            "Generic Deserialize method not implemented since XDR is not self describing",
        )))
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> DecoderResult<V::Value> {
        let value: u8 = Deserialize::deserialize(self)?;
        match value {
            1 => visitor.visit_bool(true),
            0 => visitor.visit_bool(false),
            _ => Err(EncoderError::Unknown(String::from(
                "invalid u8 when decoding bool, 0 or 1 needed",
            ))),
        }
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> DecoderResult<V::Value> {
        let res = visitor.visit_u8(self.read_u8()?);
        self.bytes_consumed += 1;
        res
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> DecoderResult<V::Value> {
        let res = visitor.visit_i8(self.read_i8()?);
        self.bytes_consumed += 1;
        res
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> DecoderResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(SeqVisitor::new(self, Some(fields.len() as u32)))
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> DecoderResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> DecoderResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqVisitor::new(self, None))
    }
}

impl<R> Read for Deserializer<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

#[derive(Debug)]
struct SeqVisitor<'a, R>
where
    R: Read,
{
    deserializer: &'a mut Deserializer<R>,
    len: Option<u32>,
}

impl<'a, 'de, R> SeqVisitor<'a, R>
where
    R: Read,
{
    fn new(de: &'a mut Deserializer<R>, size: Option<u32>) -> Self {
        SeqVisitor {
            deserializer: de,
            len: size,
        }
    }
}

impl<'de, 'a, R> de::SeqAccess<'de> for SeqVisitor<'a, R>
where
    R: Read,
{
    type Error = EncoderError;

    fn next_element_seed<V>(&mut self, seed: V) -> DecoderResult<Option<V::Value>>
    where
        V: de::DeserializeSeed<'de>,
    {
        if self.len.is_none() {
            self.len = Some(Deserialize::deserialize(&mut *self.deserializer)?);
        }
        let len = self.len.unwrap();
        if len > 0 {
            if let Some(v) = self.len.iter_mut().next() {
                *v = len - 1
            }
            let value = seed.deserialize(&mut *self.deserializer)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

impl<'de, R> de::VariantAccess<'de> for Deserializer<R>
where
    R: Read,
{
    type Error = EncoderError;

    fn newtype_variant_seed<T>(self, _seed: T) -> DecoderResult<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(EncoderError::Unknown(String::from(
            "XDR deserialize not implemented for this type",
        )))
    }

    fn unit_variant(self) -> DecoderResult<()> {
        Ok(())
    }

    fn newtype_variant<T>(self) -> DecoderResult<T>
    where
        T: de::Deserialize<'de>,
    {
        Err(EncoderError::Unknown(String::from(
            "XDR deserialize not implemented for this type",
        )))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> DecoderResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(EncoderError::Unknown(String::from(
            "XDR deserialize not implemented for this type",
        )))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> DecoderResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(EncoderError::Unknown(String::from(
            "XDR deserialize not implemented for this type",
        )))
    }
}

#[derive(Debug)]
struct VariantVisitor<'a, R>
where
    R: Read,
{
    de: &'a mut Deserializer<R>,
    style: XdrEnumType,
    variants: &'static [&'static str],
}

impl<'a, 'de, R> VariantVisitor<'a, R>
where
    R: Read,
{
    fn new(
        de: &'a mut Deserializer<R>,
        style: XdrEnumType,
        variants: &'static [&'static str],
    ) -> Self {
        VariantVisitor {
            de,
            style,
            variants,
        }
    }
}

impl<'de, 'a, R> de::EnumAccess<'de> for VariantVisitor<'a, R>
where
    R: Read,
{
    type Error = EncoderError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> DecoderResult<(V::Value, Self)>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.style {
            XdrEnumType::Union => {
                let enum_index: u32 = Deserialize::deserialize(&mut *self.de)?;
                // Clippy thinks this can be re-written as a let if but it can't because we're returning from
                // the None variant of the match and that jacks it all up.
                #[allow(clippy::useless_let_if_seq)]
                let mut union_index: u32 = (self.variants.len() - 1) as u32;
                if enum_index < self.variants.len() as u32 {
                    let ids = self
                        .variants
                        .iter()
                        .map(|x| x.parse::<u32>().unwrap())
                        .position(|x| x == enum_index);
                    union_index = match ids {
                        Some(idx) => idx as u32,
                        None => {
                            return Err(EncoderError::Unknown(String::from(
                                "Bad Index for Union, the codegen annotations are broken probably",
                            )));
                        }
                    };
                }

                let des = union_index.into_deserializer();
                let val: Result<V::Value, de::value::Error> = seed.deserialize(des);
                Ok((val.unwrap(), self))
            }
            XdrEnumType::Enum => {
                let val = seed.deserialize(&mut *self.de)?;
                Ok((val, self))
            }
        }
    }
}

impl<'de, 'a, R> de::VariantAccess<'de> for VariantVisitor<'a, R>
where
    R: Read,
{
    type Error = EncoderError;

    fn unit_variant(self) -> DecoderResult<()> {
        de::Deserialize::deserialize(self.de)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> DecoderResult<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> DecoderResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(SeqVisitor::new(self.de, Some(len as u32)))
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> DecoderResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(SeqVisitor::new(self.de, Some(fields.len() as u32)))
    }
}
