use crate::errors::{EncoderError, EncoderResult};

use byteorder::{BigEndian, WriteBytesExt};
use serde::ser;
use std::io;

macro_rules! not_implemented {
    ($($name:ident($($arg:ident: $ty:ty,)*);)*) => {
        $(fn $name<>(self, $($arg: $ty,)*) -> EncoderResult<()> {
            Err(EncoderError::Unknown(format!("Serialize Not Implemented for {}", stringify!($name))))
        })*
    }
}

pub struct Serializer<W> {
    writer: W,
}

impl<W: io::Write> Serializer<W> {
    pub fn new(writer: W) -> Self {
        Serializer { writer }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }
}

#[derive(Debug)]
pub struct MapState {
    size: usize,
    slots: Vec<u8>,
}

impl<'a, W: io::Write> ser::Serializer for &'a mut Serializer<W> {
    type Error = EncoderError;
    type Ok = ();

    type SerializeSeq = Compound<'a, W>;
    type SerializeTuple = Compound<'a, W>;
    type SerializeTupleStruct = Compound<'a, W>;
    type SerializeTupleVariant = Compound<'a, W>;
    type SerializeMap = Compound<'a, W>;
    type SerializeStruct = Compound<'a, W>;
    type SerializeStructVariant = Compound<'a, W>;

    not_implemented!(
        serialize_f32(_val: f32,);
        serialize_f64(_val: f64,);
        serialize_none();
        serialize_unit_struct(_name: &'static str,);
    );

    fn serialize_i8(self, value: i8) -> EncoderResult<()> {
        self.writer.write_i8(value).map_err(From::from)
    }

    fn serialize_i16(self, value: i16) -> EncoderResult<()> {
        self.writer
            .write_i16::<BigEndian>(value)
            .map_err(From::from)
    }

    fn serialize_i32(self, value: i32) -> EncoderResult<()> {
        self.writer
            .write_i32::<BigEndian>(value)
            .map_err(From::from)
    }

    fn serialize_i64(self, value: i64) -> EncoderResult<()> {
        self.writer
            .write_i64::<BigEndian>(value)
            .map_err(From::from)
    }

    fn serialize_u8(self, value: u8) -> EncoderResult<()> {
        self.writer.write_u8(value).map_err(From::from)
    }

    fn serialize_u16(self, value: u16) -> EncoderResult<()> {
        self.writer
            .write_u16::<BigEndian>(value)
            .map_err(From::from)
    }

    fn serialize_u32(self, value: u32) -> EncoderResult<()> {
        self.writer
            .write_u32::<BigEndian>(value)
            .map_err(From::from)
    }

    fn serialize_u64(self, value: u64) -> EncoderResult<()> {
        self.writer
            .write_u64::<BigEndian>(value)
            .map_err(From::from)
    }

    fn serialize_bytes(self, _val: &[u8]) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not yet implemented")))
    }

    fn serialize_char(self, val: char) -> EncoderResult<()> {
        self.serialize_u8(val as u8)
    }

    fn serialize_str(self, val: &str) -> EncoderResult<()> {
        self.serialize_u32(val.len() as u32).unwrap();
        let extra_bytes = 4 - val.len() % 4;
        for c in val.chars() {
            self.serialize_char(c).unwrap();
        }
        // Spec needs padding to multiple of 4
        for _ in 0..extra_bytes {
            self.serialize_u8(0 as u8).unwrap();
        }
        Ok(())
    }
    fn serialize_bool(self, v: bool) -> EncoderResult<()> {
        self.writer
            .write_u8(if v { 1 } else { 0 })
            .map_err(From::from)
    }

    fn serialize_unit(self) -> EncoderResult<()> {
        Ok(())
    }

    fn serialize_some<T>(self, _value: &T) -> EncoderResult<()>
    where
        T: ser::Serialize + ?Sized,
    {
        Err(EncoderError::Unknown(String::from("Not yet implemented")))
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> EncoderResult<()>
    where
        T: ser::Serialize + ?Sized,
    {
        Err(EncoderError::Unknown(String::from("Not yet implemented")))
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> EncoderResult<()>
    where
        T: ser::Serialize + ?Sized,
    {
        Err(EncoderError::Unknown(String::from("Not yet implemented")))
    }

    // fn serialize_seq_fixed_size(self, size: usize) -> EncoderResult<Self::SerializeSeq> {
    //     Ok(Compound {
    //         ser: self,
    //         size: None,
    //     })
    // }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> EncoderResult<Self::SerializeStruct> {
        Ok(Compound {
            ser: self,
            size: Some(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> EncoderResult<Self::SerializeMap> {
        Err(EncoderError::Unknown(String::from("Not yet implemented")))
    }

    fn serialize_unit_variant(
        self,
        _name: &str,
        variant_index: u32,
        _variant: &str,
    ) -> EncoderResult<()> {
        self.serialize_i32(variant_index as i32)
    }

    fn serialize_seq(self, len: Option<usize>) -> EncoderResult<Self::SerializeSeq> {
        self.serialize_u32(len.unwrap() as u32).unwrap();
        Ok(Compound {
            ser: self,
            size: len,
        })
    }

    fn serialize_tuple(self, len: usize) -> EncoderResult<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> EncoderResult<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> EncoderResult<Self::SerializeTupleVariant> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_idx: u32,
        variant: &'static str,
        _len: usize,
    ) -> EncoderResult<Self::SerializeStructVariant> {
        let descr_idx = variant.parse::<u32>();
        match descr_idx {
            Ok(idx) => {
                self.serialize_u32(idx).unwrap();
                Ok(Compound {
                    ser: self,
                    size: Some(idx as usize),
                })
            }
            Err(_) => {
                self.serialize_u32((variant_idx + 1) as u32).unwrap();
                Ok(Compound {
                    ser: self,
                    size: Some((variant_idx + 1) as usize),
                })
            }
        }
    }
}

pub struct Compound<'a, W: 'a> {
    ser: &'a mut Serializer<W>,
    size: Option<usize>,
}

impl<'a, W> ser::SerializeSeq for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = EncoderError;

    fn serialize_element<T>(&mut self, value: &T) -> EncoderResult<()>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> EncoderResult<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleVariant for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = EncoderError;

    fn serialize_field<T>(&mut self, _value: &T) -> EncoderResult<()>
    where
        T: ser::Serialize + ?Sized,
    {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    fn end(self) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }
}

impl<'a, W> ser::SerializeTuple for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = EncoderError;

    fn serialize_element<T>(&mut self, _value: &T) -> EncoderResult<()>
    where
        T: ser::Serialize + ?Sized,
    {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    fn end(self) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }
}

impl<'a, W> ser::SerializeMap for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = EncoderError;

    fn serialize_key<T>(&mut self, _value: &T) -> EncoderResult<()>
    where
        T: ser::Serialize + ?Sized,
    {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    fn serialize_value<T>(&mut self, value: &T) -> EncoderResult<()>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> EncoderResult<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeStruct for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = EncoderError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> EncoderResult<()>
    where
        T: ser::Serialize + ?Sized,
    {
        ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> EncoderResult<()> {
        ser::SerializeMap::end(self)
    }
}

impl<'a, W> ser::SerializeStructVariant for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = EncoderError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> EncoderResult<()>
    where
        T: ser::Serialize + ?Sized,
    {
        ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> EncoderResult<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleStruct for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = EncoderError;

    fn serialize_field<T>(&mut self, _value: &T) -> EncoderResult<()>
    where
        T: ser::Serialize + ?Sized,
    {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }

    fn end(self) -> EncoderResult<()> {
        Err(EncoderError::Unknown(String::from("Not Implemented")))
    }
}
