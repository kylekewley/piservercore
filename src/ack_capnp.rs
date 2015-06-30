// Generated by the capnpc-rust plugin to the Cap'n Proto schema compiler.
// DO NOT EDIT.
// source: ack.capnp


pub mod ack {
  #![allow(unused_imports)]
  use capnp::capability::{FromClientHook, FromTypelessPipeline};
  use capnp::{text, data, Result};
  use capnp::private::layout;
  use capnp::traits::{FromStructBuilder, FromStructReader};
  use capnp::{primitive_list, enum_list, struct_list, text_list, data_list, list_list};

  #[derive(Clone, Copy)]
  pub struct Reader<'a> { reader : layout::StructReader<'a> }

  impl <'a> ::capnp::traits::HasTypeId for Reader<'a> {
    #[inline]
    fn type_id() -> u64 { _private::TYPE_ID }
  }
  impl <'a> ::capnp::traits::FromStructReader<'a> for Reader<'a> {
    fn new(reader: ::capnp::private::layout::StructReader<'a>) -> Reader<'a> {
      Reader { reader : reader }
    }
  }

  impl <'a> ::capnp::traits::FromPointerReader<'a> for Reader<'a> {
    fn get_from_pointer(reader: &::capnp::private::layout::PointerReader<'a>) -> Result<Reader<'a>> {
      ::std::result::Result::Ok(::capnp::traits::FromStructReader::new(try!(reader.get_struct(::std::ptr::null()))))
    }
  }

  impl <'a, 'b : 'a> ::capnp::traits::CastableTo<Reader<'a>> for Reader<'b> {
    fn cast(self) -> Reader<'a> { Reader { reader : self.reader } }
  }

  impl <'a> Reader<'a> {
    pub fn borrow<'b>(&'b self) -> Reader<'b> {
      Reader { reader : self.reader}
    }

    pub fn total_size(&self) -> Result<::capnp::MessageSize> {
      self.reader.total_size()
    }
    #[inline]
    pub fn get_message_id(self) -> u64 {
      self.reader.get_data_field::<u64>(0)
    }
    #[inline]
    pub fn get_status(self) -> ::std::result::Result<::ack_capnp::ack::Status, ::capnp::NotInSchema> {
      ::capnp::traits::FromU16::from_u16(self.reader.get_data_field::<u16>(4))
    }
    #[inline]
    pub fn get_error(self) -> Result<::error_capnp::error::Reader<'a>> {
      ::capnp::traits::FromPointerReader::get_from_pointer(&self.reader.get_pointer_field(0))
    }
    pub fn has_error(&self) -> bool {
      !self.reader.get_pointer_field(0).is_null()
    }
  }

  pub struct Builder<'a> { builder : ::capnp::private::layout::StructBuilder<'a> }
  impl <'a> ::capnp::traits::HasStructSize for Builder<'a> {
    #[inline]
    fn struct_size() -> layout::StructSize { _private::STRUCT_SIZE }
  }
  impl <'a> ::capnp::traits::HasTypeId for Builder<'a> {
    #[inline]
    fn type_id() -> u64 { _private::TYPE_ID }
  }
  impl <'a> ::capnp::traits::FromStructBuilder<'a> for Builder<'a> {
    fn new(builder : ::capnp::private::layout::StructBuilder<'a>) -> Builder<'a> {
      Builder { builder : builder }
    }
  }

  impl <'a> ::capnp::traits::FromPointerBuilder<'a> for Builder<'a> {
    fn init_pointer(builder: ::capnp::private::layout::PointerBuilder<'a>, _size : u32) -> Builder<'a> {
      ::capnp::traits::FromStructBuilder::new(builder.init_struct(_private::STRUCT_SIZE))
    }
    fn get_from_pointer(builder: ::capnp::private::layout::PointerBuilder<'a>) -> Result<Builder<'a>> {
      ::std::result::Result::Ok(::capnp::traits::FromStructBuilder::new(try!(builder.get_struct(_private::STRUCT_SIZE, ::std::ptr::null()))))
    }
  }

  impl <'a> ::capnp::traits::SetPointerBuilder<Builder<'a>> for Reader<'a> {
    fn set_pointer_builder<'b>(pointer : ::capnp::private::layout::PointerBuilder<'b>, value : Reader<'a>) -> Result<()> { pointer.set_struct(&value.reader) }
  }

  impl <'a, 'b : 'a> ::capnp::traits::CastableTo<Builder<'a>> for Builder<'b> {
    fn cast(self) -> Builder<'a> { Builder { builder : self.builder } }
  }

  impl <'a> Builder<'a> {
    pub fn as_reader(self) -> Reader<'a> {
      ::capnp::traits::FromStructReader::new(self.builder.as_reader())
    }
    pub fn borrow<'b>(&'b mut self) -> Builder<'b> {
      Builder { builder : self.builder}
    }

    pub fn total_size(&self) -> Result<::capnp::MessageSize> {
      self.builder.as_reader().total_size()
    }
    #[inline]
    pub fn get_message_id(self) -> u64 {
      self.builder.get_data_field::<u64>(0)
    }
    #[inline]
    pub fn set_message_id(&mut self, value : u64)  {
      self.builder.set_data_field::<u64>(0, value);
    }
    #[inline]
    pub fn get_status(self) -> ::std::result::Result<::ack_capnp::ack::Status, ::capnp::NotInSchema> {
      ::capnp::traits::FromU16::from_u16(self.builder.get_data_field::<u16>(4))
    }
    #[inline]
    pub fn set_status(&mut self, value : ::ack_capnp::ack::Status)  {
      self.builder.set_data_field::<u16>(4, value as u16)
    }
    #[inline]
    pub fn get_error(self) -> Result<::error_capnp::error::Builder<'a>> {
      ::capnp::traits::FromPointerBuilder::get_from_pointer(self.builder.get_pointer_field(0))
    }
    #[inline]
    pub fn set_error(&mut self, value : ::error_capnp::error::Reader) -> Result<()> {
      ::capnp::traits::SetPointerBuilder::set_pointer_builder(self.builder.get_pointer_field(0), value)
    }
    #[inline]
    pub fn init_error(self, ) -> ::error_capnp::error::Builder<'a> {
      ::capnp::traits::FromPointerBuilder::init_pointer(self.builder.get_pointer_field(0), 0)
    }
    pub fn has_error(&self) -> bool {
      !self.builder.get_pointer_field(0).is_null()
    }
  }

  pub struct Pipeline { _typeless : ::capnp::any_pointer::Pipeline }
  impl FromTypelessPipeline for Pipeline {
    fn new(typeless : ::capnp::any_pointer::Pipeline) -> Pipeline {
      Pipeline { _typeless : typeless }
    }
  }
  impl Pipeline {
    pub fn get_error(&self) -> ::error_capnp::error::Pipeline {
      FromTypelessPipeline::new(self._typeless.get_pointer_field(0))
    }
  }
  mod _private {
    use capnp::private::layout;
    pub const STRUCT_SIZE : layout::StructSize = layout::StructSize { data : 2, pointers : 1 };
    pub const TYPE_ID: u64 = 0xa4df2f25bc49a3a6;
  }

  #[repr(u16)]
  #[derive(Clone, Copy, PartialEq)]
  pub enum Status {
    Success = 0,
    Resend = 1,
    Failure = 2,
  }
  impl ::capnp::traits::FromU16 for Status {
    #[inline]
    fn from_u16(value : u16) -> ::std::result::Result<Status, ::capnp::NotInSchema> {
      match value {
        0 => ::std::result::Result::Ok(Status::Success),
        1 => ::std::result::Result::Ok(Status::Resend),
        2 => ::std::result::Result::Ok(Status::Failure),
        n => ::std::result::Result::Err(::capnp::NotInSchema(n)),
      }
    }
  }
  impl ::capnp::traits::ToU16 for Status {
    #[inline]
    fn to_u16(self) -> u16 { self as u16 }
  }
  impl ::capnp::traits::HasTypeId for Status {
    #[inline]
    fn type_id() -> u64 { 0xdcf57933ccbd76e8u64 }
  }
}