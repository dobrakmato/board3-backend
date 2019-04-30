use serde::{ser, Serialize};
use crate::error::{Error, Result};
use std::fmt::Display;

pub struct Serializer {
    output: Vec<u8>,
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
    where T: Serialize,
{
    let mut serializer = Serializer {
        output: Vec::new(),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    /* for booleans 0 = false, 1 = true */
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output.push(if v { 1 } else { 0 });
        Ok(())
    }

    /* signed integers are transparently encoded as unsigned */
    fn serialize_i8(self, v: i8) -> Result<()> {
        return self.serialize_u8(v as u8);
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        return self.serialize_u16(v as u16);
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        return self.serialize_u32(v as u32);
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        return self.serialize_u64(v as u64);
    }

    /* little endian for unsigned multi-byte integers */
    fn serialize_u8(self, v: u8) -> Result<()> {
        self.output.push(v);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.output.push((v & 0xff) as u8);
        self.output.push(((v >> 8) & 0xff) as u8);
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.output.push((v & 0xff) as u8);
        self.output.push(((v >> 8) & 0xff) as u8);
        self.output.push(((v >> 16) & 0xff) as u8);
        self.output.push(((v >> 24) & 0xff) as u8);
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output.push((v & 0xff) as u8);
        self.output.push(((v >> 8) & 0xff) as u8);
        self.output.push(((v >> 16) & 0xff) as u8);
        self.output.push(((v >> 24) & 0xff) as u8);
        self.output.push(((v >> 32) & 0xff) as u8);
        self.output.push(((v >> 40) & 0xff) as u8);
        self.output.push(((v >> 48) & 0xff) as u8);
        self.output.push(((v >> 56) & 0xff) as u8);
        Ok(())
    }

    /* floating types are unsupported */
    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_u32(v.to_bits())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.serialize_u64(v.to_bits())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        return self.serialize_u32(u32::from(v));
    }


    fn serialize_str(self, v: &str) -> Result<()> {
        if v.len() >= (1 << 16) {
            return Err(Error::Message("string longer than allowed".to_string()));
        }

        self.serialize_u16(v.len() as u16)?;
        self.output.extend_from_slice(v.as_bytes());
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        if v.len() >= (1 << 16) {
            return Err(Error::Message("bytes longer than allowed".to_string()));
        }

        self.serialize_u16(v.len() as u16)?;
        self.output.extend_from_slice(v);
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_bool(false)?;
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()> where T: Serialize {
        self.serialize_bool(true)?;
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(self, _name: &'static str, variant_index: u32, _variant: &'static str) -> Result<()> {
        if variant_index >= (1 << 8) {
            return Err(Error::Message("variant index is greater than allowed".to_string()));
        }

        self.serialize_u8(variant_index as u8)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()> where T: Serialize {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(self, _name: &'static str, variant_index: u32, _variant: &'static str, value: &T) -> Result<()> where
        T: Serialize {
        if variant_index >= (1 << 8) {
            return Err(Error::Message("variant index is greater than allowed".to_string()));
        }

        self.serialize_u8(variant_index as u8)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if len.is_none() { return Err(Error::Message("seq of unknown len is not supported".to_string())); }
        if len.unwrap() >= (1 << 16) {
            return Err(Error::Message("seq longer than allowed".to_string()));
        }

        self.serialize_u16(len.unwrap() as u16)?;
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(self, _name: &'static str, variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant> {
        if variant_index >= (1 << 8) {
            return Err(Error::Message("variant index is greater than allowed".to_string()));
        }

        self.serialize_u8(variant_index as u8)?;

        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        return Err(Error::Message("serialize_map is not supported".to_string()));
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(self, _name: &'static str, variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant> {
        if variant_index >= (1 << 8) {
            return Err(Error::Message("variant index is greater than allowed".to_string()));
        }

        self.serialize_u8(variant_index as u8)?;
        Ok(self)
    }

    fn collect_str<T: ?Sized>(self, _value: &T) -> Result<()> where T: Display {
        return Err(Error::Message("collect_str is not supported".to_string()));
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()> where
        T: Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()> where
        T: Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()> where
        T: Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()> where T: Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()> where T: Serialize {
        key.serialize(&mut **self)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()> where T: Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()> where
        T: Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()> where
        T: Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}