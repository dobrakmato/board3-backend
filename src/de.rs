use serde::{Deserialize, de};
use crate::error::{Error, Result};
use serde::de::{Visitor, SeqAccess, DeserializeSeed, EnumAccess, VariantAccess};

pub struct Deserializer<'de> {
    input: &'de [u8],
    pos: usize,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer { input, pos: 0 }
    }
}

pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T> where T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(s);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

impl<'de> Deserializer<'de> {
    fn read_u8(&mut self) -> Result<u8> {
        let val = self.input[self.pos];
        self.pos += 1;
        Ok(val)
    }

    fn read_u16(&mut self) -> Result<u16> {
        let a = self.read_u8()? as u16;
        let b = self.read_u8()? as u16;

        Ok((b << 8) | a)
    }

    fn read_u32(&mut self) -> Result<u32> {
        let a = self.read_u8()? as u32;
        let b = self.read_u8()? as u32;
        let c = self.read_u8()? as u32;
        let d = self.read_u8()? as u32;

        Ok(((d << 24) | (c << 16) | b << 8) | a)
    }

    fn read_u64(&mut self) -> Result<u64> {
        let a = self.read_u8()? as u64;
        let b = self.read_u8()? as u64;
        let c = self.read_u8()? as u64;
        let d = self.read_u8()? as u64;
        let e = self.read_u8()? as u64;
        let f = self.read_u8()? as u64;
        let g = self.read_u8()? as u64;
        let h = self.read_u8()? as u64;

        Ok(((h << 56) | (g << 48) | (f << 40) | (e << 32) | (d << 24) | (c << 16) | b << 8) | a)
    }

    fn read_bytes(&mut self, length: usize) -> Result<&'de [u8]> {
        let ptr = &self.input[self.pos..self.pos + length];
        self.pos += length;
        Ok(ptr)
    }

    fn read_bool(&mut self) -> Result<bool> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::Message("Incorrect integer value provided for boolean".to_string()))
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(Error::Message("deserialize_any nto supported!".to_string()))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_bool(self.read_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_i8(self.read_u8()? as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_i16(self.read_u16()? as i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_i32(self.read_u32()? as i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_i64(self.read_u64()? as i64)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_u8(self.read_u8()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_u16(self.read_u16()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_u32(self.read_u32()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_u64(self.read_u64()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_f32(f32::from_bits(self.read_u32()?))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_f64(f64::from_bits(self.read_u64()?))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let v = self.read_u32()?;
        let c = std::char::from_u32(v);
        match c {
            Some(t) => visitor.visit_char(t),
            None => Err(Error::Message("Invalid utf-8 string".to_string()))
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let length = self.read_u16()?;
        let bytes = self.read_bytes(length as usize)?;

        let maybe_str = std::str::from_utf8(bytes);

        match maybe_str {
            Ok(t) => visitor.visit_borrowed_str(t),
            Err(err) => Err(Error::Message(err.to_string()))
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let length = self.read_u16()?;
        let bytes = self.read_bytes(length as usize)?;

        let maybe_str = std::str::from_utf8(bytes);

        match maybe_str {
            Ok(t) => visitor.visit_string(t.to_string()),
            Err(err) => Err(Error::Message(err.to_string()))
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let length = self.read_u16()?;
        let bytes = self.read_bytes(length as usize)?;

        visitor.visit_borrowed_bytes(bytes)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let length = self.read_u16()?;
        let bytes = self.read_bytes(length as usize)?;

        visitor.visit_byte_buf(bytes.to_vec())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let present = self.read_bool()?;
        match present {
            true => visitor.visit_some(self),
            false => visitor.visit_none(),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let len = self.read_u16()?;
        visitor.visit_seq(Seq::with_len(&mut self, len as usize))
    }

    fn deserialize_tuple<V>(mut self, len: usize, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_seq(Seq::with_len(&mut self, len as usize))
    }

    fn deserialize_tuple_struct<V>(mut self, _name: &'static str, len: usize, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_seq(Seq::with_len(&mut self, len as usize))
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(Error::Message("deserialize_map is unsupported!".to_string()))
    }

    fn deserialize_struct<V>(mut self, _name: &'static str, fields: &'static [&'static str], visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_seq(Seq::with_len(&mut self, fields.len()))
    }

    fn deserialize_enum<V>(self, _name: &'static str, _variants: &'static [&'static str], visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_enum(Enum::new(self))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_u8(self.read_u8()?)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(Error::Message("deserialize_ignored_any is unsupported!".to_string()))
    }
}

struct Seq<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: usize,
}

impl<'de, 'a> Seq<'a, 'de> {
    fn with_len(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        Seq { de, len }
    }
}

impl<'de, 'a> SeqAccess<'de> for Seq<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>> where T: DeserializeSeed<'de> {
        if self.len == 0 { return Ok(None); }
        self.len -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }
}


struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)> where V: DeserializeSeed<'de> {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value> where T: DeserializeSeed<'de> {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        de::Deserializer::deserialize_tuple(self.de, len, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        de::Deserializer::deserialize_tuple(self.de, fields.len(), visitor)
    }
}