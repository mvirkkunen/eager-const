use serde::{ser, Serialize};
use std::fmt;

#[derive(Debug)]
pub struct Error(String);

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error(msg.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.0
    }
}

type Result<T> = std::result::Result<T, Error>;

struct Serializer {
    output: String,
}

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: String::new(),
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

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output += if v { "true" } else { "false" };
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.output += "'";
        self.output.push(v);
        self.output += "'";
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.output += "\"";
        self.output += v;
        self.output += "\"";
        Ok(())
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.output += "b\"";

        self.output += "\"";
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> Result<()> {
        self.output += "None";
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output += "Some(";
        value.serialize(&mut *self)?;
        self.output += ")";
        Ok(())
    }

    fn serialize_unit(self) -> Result<()> {
        self.output += "()";
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        self.output += name;
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.output += name;
        self.output += "::";
        self.output += variant;
        Ok(())
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output += name;
        self.output += "(";
        value.serialize(&mut *self)?;
        self.output += ")";
        Ok(())
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output += name;
        self.output += "::";
        self.output += variant;
        self.output += "(";
        value.serialize(&mut *self)?;
        self.output += ")";
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output += "&[";
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        self.output += "(";
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.output += name;
        self.output += "(";
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.output += name;
        self.output += "::";
        self.output += variant;
        self.output += "(";
        variant.serialize(&mut *self)?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error("serialize_map: unsupported".into()))
    }

    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        self.output += name;
        self.output += "{";
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.output += name;
        self.output += "::";
        self.output += variant;
        self.output += "{";
        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        self.output += ",";
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.output += "]";
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        self.output += ",";
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.output += ")";
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        self.output += ",";
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.output += ")";
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        self.output += ",";
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.output += ")";
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
    }
    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
    }

    fn end(self) -> Result<()> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        self.output += ":";
        value.serialize(&mut **self)?;
        self.output += ",";
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.output += "}";
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        self.output += ":";
        value.serialize(&mut **self)?;
        self.output += ",";
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.output += "}";
        Ok(())
    }
}
