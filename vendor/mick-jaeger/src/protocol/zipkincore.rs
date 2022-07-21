// Copied from https://github.com/open-telemetry/opentelemetry-rust/blob/master/opentelemetry-jaeger/src/thrift/jaeger.rs
// Covered by the license of https://github.com/open-telemetry/opentelemetry-rust (Apache2)

// Autogenerated by Thrift Compiler (0.13.0)
// DO NOT EDIT UNLESS YOU ARE SURE THAT YOU KNOW WHAT YOU ARE DOING

#![allow(unused_imports)]
#![allow(unused_extern_crates)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::too_many_arguments, clippy::type_complexity))]
#![cfg_attr(rustfmt, rustfmt_skip)]

extern crate thrift;

use thrift::OrderedFloat;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::{From, TryFrom};
use std::default::Default;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use thrift::{ApplicationError, ApplicationErrorKind, ProtocolError, ProtocolErrorKind, TThriftClient};
use thrift::protocol::{TFieldIdentifier, TListIdentifier, TMapIdentifier, TMessageIdentifier, TMessageType, TInputProtocol, TOutputProtocol, TSetIdentifier, TStructIdentifier, TType};
use thrift::protocol::field_id;
use thrift::protocol::verify_expected_message_type;
use thrift::protocol::verify_expected_sequence_number;
use thrift::protocol::verify_expected_service_call;
use thrift::protocol::verify_required_field_exists;
use thrift::server::TProcessor;

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AnnotationType {
  Bool = 0,
  Bytes = 1,
  I16 = 2,
  I32 = 3,
  I64 = 4,
  Double = 5,
  String = 6,
}

impl AnnotationType {
  pub fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    o_prot.write_i32(*self as i32)
  }
  pub fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<AnnotationType> {
    let enum_value = i_prot.read_i32()?;
    AnnotationType::try_from(enum_value)  }
}

impl TryFrom<i32> for AnnotationType {
  type Error = thrift::Error;  fn try_from(i: i32) -> Result<Self, Self::Error> {
    match i {
      0 => Ok(AnnotationType::Bool),
      1 => Ok(AnnotationType::Bytes),
      2 => Ok(AnnotationType::I16),
      3 => Ok(AnnotationType::I32),
      4 => Ok(AnnotationType::I64),
      5 => Ok(AnnotationType::Double),
      6 => Ok(AnnotationType::String),
      _ => {
        Err(
          thrift::Error::Protocol(
            ProtocolError::new(
              ProtocolErrorKind::InvalidData,
              format!("cannot convert enum constant {} to AnnotationType", i)
            )
          )
        )
      },
    }
  }
}

//
// Endpoint
//

/// Indicates the network context of a service recording an annotation with two
/// exceptions.
///
/// When a BinaryAnnotation, and key is CLIENT_ADDR or SERVER_ADDR,
/// the endpoint indicates the source or destination of an RPC. This exception
/// allows zipkin to display network context of uninstrumented services, or
/// clients such as web browsers.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Endpoint {
  /// IPv4 host address packed into 4 bytes.
  ///
  /// Ex for the ip 1.2.3.4, it would be (1 << 24) | (2 << 16) | (3 << 8) | 4
  pub ipv4: Option<i32>,
  /// IPv4 port
  ///
  /// Note: this is to be treated as an unsigned integer, so watch for negatives.
  ///
  /// Conventionally, when the port isn't known, port = 0.
  pub port: Option<i16>,
  /// Service name in lowercase, such as "memcache" or "zipkin-web"
  ///
  /// Conventionally, when the service name isn't known, service_name = "unknown".
  pub service_name: Option<String>,
  /// IPv6 host address packed into 16 bytes. Ex Inet6Address.getBytes()
  pub ipv6: Option<Vec<u8>>,
}

impl Endpoint {
  pub fn new<F1, F2, F3, F4>(ipv4: F1, port: F2, service_name: F3, ipv6: F4) -> Endpoint where F1: Into<Option<i32>>, F2: Into<Option<i16>>, F3: Into<Option<String>>, F4: Into<Option<Vec<u8>>> {
    Endpoint {
      ipv4: ipv4.into(),
      port: port.into(),
      service_name: service_name.into(),
      ipv6: ipv6.into(),
    }
  }
  pub fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Endpoint> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i32> = Some(0);
    let mut f_2: Option<i16> = Some(0);
    let mut f_3: Option<String> = Some("".to_owned());
    let mut f_4: Option<Vec<u8>> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_i32()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_i16()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_string()?;
          f_3 = Some(val);
        },
        4 => {
          let val = i_prot.read_bytes()?;
          f_4 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Endpoint {
      ipv4: f_1,
      port: f_2,
      service_name: f_3,
      ipv6: f_4,
    };
    Ok(ret)
  }
  pub fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Endpoint");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.ipv4 {
      o_prot.write_field_begin(&TFieldIdentifier::new("ipv4", TType::I32, 1))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(fld_var) = self.port {
      o_prot.write_field_begin(&TFieldIdentifier::new("port", TType::I16, 2))?;
      o_prot.write_i16(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(ref fld_var) = self.service_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("service_name", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(ref fld_var) = self.ipv6 {
      o_prot.write_field_begin(&TFieldIdentifier::new("ipv6", TType::String, 4))?;
      o_prot.write_bytes(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

impl Default for Endpoint {
  fn default() -> Self {
    Endpoint{
      ipv4: Some(0),
      port: Some(0),
      service_name: Some("".to_owned()),
      ipv6: Some(Vec::new()),
    }
  }
}

//
// Annotation
//

/// An annotation is similar to a log statement. It includes a host field which
/// allows these events to be attributed properly, and also aggregatable.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Annotation {
  /// Microseconds from epoch.
  ///
  /// This value should use the most precise value possible. For example,
  /// gettimeofday or syncing nanoTime against a tick of currentTimeMillis.
  pub timestamp: Option<i64>,
  pub value: Option<String>,
  /// Always the host that recorded the event. By specifying the host you allow
  /// rollup of all events (such as client requests to a service) by IP address.
  pub host: Option<Endpoint>,
}

impl Annotation {
  pub fn new<F1, F2, F3>(timestamp: F1, value: F2, host: F3) -> Annotation where F1: Into<Option<i64>>, F2: Into<Option<String>>, F3: Into<Option<Endpoint>> {
    Annotation {
      timestamp: timestamp.into(),
      value: value.into(),
      host: host.into(),
    }
  }
  pub fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Annotation> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_2: Option<String> = Some("".to_owned());
    let mut f_3: Option<Endpoint> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_i64()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_string()?;
          f_2 = Some(val);
        },
        3 => {
          let val = Endpoint::read_from_in_protocol(i_prot)?;
          f_3 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Annotation {
      timestamp: f_1,
      value: f_2,
      host: f_3,
    };
    Ok(ret)
  }
  pub fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Annotation");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.timestamp {
      o_prot.write_field_begin(&TFieldIdentifier::new("timestamp", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(ref fld_var) = self.value {
      o_prot.write_field_begin(&TFieldIdentifier::new("value", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(ref fld_var) = self.host {
      o_prot.write_field_begin(&TFieldIdentifier::new("host", TType::Struct, 3))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

impl Default for Annotation {
  fn default() -> Self {
    Annotation{
      timestamp: Some(0),
      value: Some("".to_owned()),
      host: None,
    }
  }
}

//
// BinaryAnnotation
//

/// Binary annotations are tags applied to a Span to give it context. For
/// example, a binary annotation of "http.uri" could the path to a resource in a
/// RPC call.
///
/// Binary annotations of type STRING are always queryable, though more a
/// historical implementation detail than a structural concern.
///
/// Binary annotations can repeat, and vary on the host. Similar to Annotation,
/// the host indicates who logged the event. This allows you to tell the
/// difference between the client and server side of the same key. For example,
/// the key "http.uri" might be different on the client and server side due to
/// rewriting, like "/api/v1/myresource" vs "/myresource. Via the host field,
/// you can see the different points of view, which often help in debugging.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BinaryAnnotation {
  pub key: Option<String>,
  pub value: Option<Vec<u8>>,
  pub annotation_type: Option<AnnotationType>,
  /// The host that recorded tag, which allows you to differentiate between
  /// multiple tags with the same key. There are two exceptions to this.
  ///
  /// When the key is CLIENT_ADDR or SERVER_ADDR, host indicates the source or
  /// destination of an RPC. This exception allows zipkin to display network
  /// context of uninstrumented services, or clients such as web browsers.
  pub host: Option<Endpoint>,
}

impl BinaryAnnotation {
  pub fn new<F1, F2, F3, F4>(key: F1, value: F2, annotation_type: F3, host: F4) -> BinaryAnnotation where F1: Into<Option<String>>, F2: Into<Option<Vec<u8>>>, F3: Into<Option<AnnotationType>>, F4: Into<Option<Endpoint>> {
    BinaryAnnotation {
      key: key.into(),
      value: value.into(),
      annotation_type: annotation_type.into(),
      host: host.into(),
    }
  }
  pub fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<BinaryAnnotation> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = Some("".to_owned());
    let mut f_2: Option<Vec<u8>> = Some(Vec::new());
    let mut f_3: Option<AnnotationType> = None;
    let mut f_4: Option<Endpoint> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_string()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_bytes()?;
          f_2 = Some(val);
        },
        3 => {
          let val = AnnotationType::read_from_in_protocol(i_prot)?;
          f_3 = Some(val);
        },
        4 => {
          let val = Endpoint::read_from_in_protocol(i_prot)?;
          f_4 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = BinaryAnnotation {
      key: f_1,
      value: f_2,
      annotation_type: f_3,
      host: f_4,
    };
    Ok(ret)
  }
  pub fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("BinaryAnnotation");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.key {
      o_prot.write_field_begin(&TFieldIdentifier::new("key", TType::String, 1))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(ref fld_var) = self.value {
      o_prot.write_field_begin(&TFieldIdentifier::new("value", TType::String, 2))?;
      o_prot.write_bytes(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(ref fld_var) = self.annotation_type {
      o_prot.write_field_begin(&TFieldIdentifier::new("annotation_type", TType::I32, 3))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(ref fld_var) = self.host {
      o_prot.write_field_begin(&TFieldIdentifier::new("host", TType::Struct, 4))?;
      fld_var.write_to_out_protocol(o_prot)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

impl Default for BinaryAnnotation {
  fn default() -> Self {
    BinaryAnnotation{
      key: Some("".to_owned()),
      value: Some(Vec::new()),
      annotation_type: None,
      host: None,
    }
  }
}

//
// Span
//

/// A trace is a series of spans (often RPC calls) which form a latency tree.
///
/// The root span is where trace_id = id and parent_id = Nil. The root span is
/// usually the longest interval in the trace, starting with a SERVER_RECV
/// annotation and ending with a SERVER_SEND.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Span {
  pub trace_id: Option<i64>,
  /// Span name in lowercase, rpc method for example
  ///
  /// Conventionally, when the span name isn't known, name = "unknown".
  pub name: Option<String>,
  pub id: Option<i64>,
  pub parent_id: Option<i64>,
  pub annotations: Option<Vec<Annotation>>,
  pub binary_annotations: Option<Vec<BinaryAnnotation>>,
  pub debug: Option<bool>,
  /// Microseconds from epoch of the creation of this span.
  ///
  /// This value should be set directly by instrumentation, using the most
  /// precise value possible. For example, gettimeofday or syncing nanoTime
  /// against a tick of currentTimeMillis.
  ///
  /// For compatibilty with instrumentation that precede this field, collectors
  /// or span stores can derive this via Annotation.timestamp.
  /// For example, SERVER_RECV.timestamp or CLIENT_SEND.timestamp.
  ///
  /// This field is optional for compatibility with old data: first-party span
  /// stores are expected to support this at time of introduction.
  pub timestamp: Option<i64>,
  /// Measurement of duration in microseconds, used to support queries.
  ///
  /// This value should be set directly, where possible. Doing so encourages
  /// precise measurement decoupled from problems of clocks, such as skew or NTP
  /// updates causing time to move backwards.
  ///
  /// For compatibilty with instrumentation that precede this field, collectors
  /// or span stores can derive this by subtracting Annotation.timestamp.
  /// For example, SERVER_SEND.timestamp - SERVER_RECV.timestamp.
  ///
  /// If this field is persisted as unset, zipkin will continue to work, except
  /// duration query support will be implementation-specific. Similarly, setting
  /// this field non-atomically is implementation-specific.
  ///
  /// This field is i64 vs i32 to support spans longer than 35 minutes.
  pub duration: Option<i64>,
  /// Optional unique 8-byte additional identifier for a trace. If non zero, this
  /// means the trace uses 128 bit traceIds instead of 64 bit.
  pub trace_id_high: Option<i64>,
}

impl Span {
  pub fn new<F1, F3, F4, F5, F6, F8, F9, F10, F11, F12>(trace_id: F1, name: F3, id: F4, parent_id: F5, annotations: F6, binary_annotations: F8, debug: F9, timestamp: F10, duration: F11, trace_id_high: F12) -> Span where F1: Into<Option<i64>>, F3: Into<Option<String>>, F4: Into<Option<i64>>, F5: Into<Option<i64>>, F6: Into<Option<Vec<Annotation>>>, F8: Into<Option<Vec<BinaryAnnotation>>>, F9: Into<Option<bool>>, F10: Into<Option<i64>>, F11: Into<Option<i64>>, F12: Into<Option<i64>> {
    Span {
      trace_id: trace_id.into(),
      name: name.into(),
      id: id.into(),
      parent_id: parent_id.into(),
      annotations: annotations.into(),
      binary_annotations: binary_annotations.into(),
      debug: debug.into(),
      timestamp: timestamp.into(),
      duration: duration.into(),
      trace_id_high: trace_id_high.into(),
    }
  }
  pub fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Span> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = Some(0);
    let mut f_3: Option<String> = Some("".to_owned());
    let mut f_4: Option<i64> = Some(0);
    let mut f_5: Option<i64> = None;
    let mut f_6: Option<Vec<Annotation>> = Some(Vec::new());
    let mut f_8: Option<Vec<BinaryAnnotation>> = Some(Vec::new());
    let mut f_9: Option<bool> = None;
    let mut f_10: Option<i64> = None;
    let mut f_11: Option<i64> = None;
    let mut f_12: Option<i64> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_i64()?;
          f_1 = Some(val);
        },
        3 => {
          let val = i_prot.read_string()?;
          f_3 = Some(val);
        },
        4 => {
          let val = i_prot.read_i64()?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_i64()?;
          f_5 = Some(val);
        },
        6 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<Annotation> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_0 = Annotation::read_from_in_protocol(i_prot)?;
            val.push(list_elem_0);
          }
          i_prot.read_list_end()?;
          f_6 = Some(val);
        },
        8 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<BinaryAnnotation> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_1 = BinaryAnnotation::read_from_in_protocol(i_prot)?;
            val.push(list_elem_1);
          }
          i_prot.read_list_end()?;
          f_8 = Some(val);
        },
        9 => {
          let val = i_prot.read_bool()?;
          f_9 = Some(val);
        },
        10 => {
          let val = i_prot.read_i64()?;
          f_10 = Some(val);
        },
        11 => {
          let val = i_prot.read_i64()?;
          f_11 = Some(val);
        },
        12 => {
          let val = i_prot.read_i64()?;
          f_12 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = Span {
      trace_id: f_1,
      name: f_3,
      id: f_4,
      parent_id: f_5,
      annotations: f_6,
      binary_annotations: f_8,
      debug: f_9,
      timestamp: f_10,
      duration: f_11,
      trace_id_high: f_12,
    };
    Ok(ret)
  }
  pub fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Span");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(fld_var) = self.trace_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("trace_id", TType::I64, 1))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(ref fld_var) = self.name {
      o_prot.write_field_begin(&TFieldIdentifier::new("name", TType::String, 3))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(fld_var) = self.id {
      o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::I64, 4))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(fld_var) = self.parent_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("parent_id", TType::I64, 5))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(ref fld_var) = self.annotations {
      o_prot.write_field_begin(&TFieldIdentifier::new("annotations", TType::List, 6))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::Struct, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
        o_prot.write_list_end()?;
      }
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(ref fld_var) = self.binary_annotations {
      o_prot.write_field_begin(&TFieldIdentifier::new("binary_annotations", TType::List, 8))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::Struct, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
        o_prot.write_list_end()?;
      }
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(fld_var) = self.debug {
      o_prot.write_field_begin(&TFieldIdentifier::new("debug", TType::Bool, 9))?;
      o_prot.write_bool(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(fld_var) = self.timestamp {
      o_prot.write_field_begin(&TFieldIdentifier::new("timestamp", TType::I64, 10))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(fld_var) = self.duration {
      o_prot.write_field_begin(&TFieldIdentifier::new("duration", TType::I64, 11))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    if let Some(fld_var) = self.trace_id_high {
      o_prot.write_field_begin(&TFieldIdentifier::new("trace_id_high", TType::I64, 12))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

impl Default for Span {
  fn default() -> Self {
    Span{
      trace_id: Some(0),
      name: Some("".to_owned()),
      id: Some(0),
      parent_id: Some(0),
      annotations: Some(Vec::new()),
      binary_annotations: Some(Vec::new()),
      debug: Some(false),
      timestamp: Some(0),
      duration: Some(0),
      trace_id_high: Some(0),
    }
  }
}

//
// Response
//

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Response {
  pub ok: bool,
}

impl Response {
  pub fn new(ok: bool) -> Response {
    Response {
      ok,

    }
  }
  pub fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<Response> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<bool> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_bool()?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("Response.ok", &f_1)?;
    let ret = Response {
      ok: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  pub fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("Response");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("ok", TType::Bool, 1))?;
    o_prot.write_bool(self.ok)?;
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

pub const C_L_I_E_N_T_S_E_N_D: &str = "cs";

pub const C_L_I_E_N_T_R_E_C_V: &str = "cr";

pub const S_E_R_V_E_R_S_E_N_D: &str = "ss";

pub const S_E_R_V_E_R_R_E_C_V: &str = "sr";

pub const M_E_S_S_A_G_E_S_E_N_D: &str = "ms";

pub const M_E_S_S_A_G_E_R_E_C_V: &str = "mr";

pub const W_I_R_E_S_E_N_D: &str = "ws";

pub const W_I_R_E_R_E_C_V: &str = "wr";

pub const C_L_I_E_N_T_S_E_N_D_F_R_A_G_M_E_N_T: &str = "csf";

pub const C_L_I_E_N_T_R_E_C_V_F_R_A_G_M_E_N_T: &str = "crf";

pub const S_E_R_V_E_R_S_E_N_D_F_R_A_G_M_E_N_T: &str = "ssf";

pub const S_E_R_V_E_R_R_E_C_V_F_R_A_G_M_E_N_T: &str = "srf";

pub const L_O_C_A_L_C_O_M_P_O_N_E_N_T: &str = "lc";

pub const C_L_I_E_N_T_A_D_D_R: &str = "ca";

pub const S_E_R_V_E_R_A_D_D_R: &str = "sa";

pub const M_E_S_S_A_G_E_A_D_D_R: &str = "ma";

//
// ZipkinCollector service client
//

pub trait TZipkinCollectorSyncClient {
  fn submit_zipkin_batch(&mut self, spans: Vec<Span>) -> thrift::Result<Vec<Response>>;
}

pub trait TZipkinCollectorSyncClientMarker {}

pub struct ZipkinCollectorSyncClient<IP, OP> where IP: TInputProtocol, OP: TOutputProtocol {
  _i_prot: IP,
  _o_prot: OP,
  _sequence_number: i32,
}

impl <IP, OP> ZipkinCollectorSyncClient<IP, OP> where IP: TInputProtocol, OP: TOutputProtocol {
  pub fn new(input_protocol: IP, output_protocol: OP) -> ZipkinCollectorSyncClient<IP, OP> {
    ZipkinCollectorSyncClient { _i_prot: input_protocol, _o_prot: output_protocol, _sequence_number: 0 }
  }
}

impl <IP, OP> TThriftClient for ZipkinCollectorSyncClient<IP, OP> where IP: TInputProtocol, OP: TOutputProtocol {
  fn i_prot_mut(&mut self) -> &mut dyn TInputProtocol { &mut self._i_prot }
  fn o_prot_mut(&mut self) -> &mut dyn TOutputProtocol { &mut self._o_prot }
  fn sequence_number(&self) -> i32 { self._sequence_number }
  fn increment_sequence_number(&mut self) -> i32 { self._sequence_number += 1; self._sequence_number }
}

impl <IP, OP> TZipkinCollectorSyncClientMarker for ZipkinCollectorSyncClient<IP, OP> where IP: TInputProtocol, OP: TOutputProtocol {}

impl <C: TThriftClient + TZipkinCollectorSyncClientMarker> TZipkinCollectorSyncClient for C {
  fn submit_zipkin_batch(&mut self, spans: Vec<Span>) -> thrift::Result<Vec<Response>> {
    (
      {
        self.increment_sequence_number();
        let message_ident = TMessageIdentifier::new("submitZipkinBatch", TMessageType::Call, self.sequence_number());
        let call_args = ZipkinCollectorSubmitZipkinBatchArgs { spans };
        self.o_prot_mut().write_message_begin(&message_ident)?;
        call_args.write_to_out_protocol(self.o_prot_mut())?;
        self.o_prot_mut().write_message_end()?;
        self.o_prot_mut().flush()
      }
    )?;
    {
      let message_ident = self.i_prot_mut().read_message_begin()?;
      verify_expected_sequence_number(self.sequence_number(), message_ident.sequence_number)?;
      verify_expected_service_call("submitZipkinBatch", &message_ident.name)?;
      if message_ident.message_type == TMessageType::Exception {
        let remote_error = thrift::Error::read_application_error_from_in_protocol(self.i_prot_mut())?;
        self.i_prot_mut().read_message_end()?;
        return Err(thrift::Error::Application(remote_error))
      }
      verify_expected_message_type(TMessageType::Reply, message_ident.message_type)?;
      let result = ZipkinCollectorSubmitZipkinBatchResult::read_from_in_protocol(self.i_prot_mut())?;
      self.i_prot_mut().read_message_end()?;
      result.ok_or()
    }
  }
}

//
// ZipkinCollector service processor
//

pub trait ZipkinCollectorSyncHandler {
  fn handle_submit_zipkin_batch(&self, spans: Vec<Span>) -> thrift::Result<Vec<Response>>;
}

pub struct ZipkinCollectorSyncProcessor<H: ZipkinCollectorSyncHandler> {
  handler: H,
}

impl <H: ZipkinCollectorSyncHandler> ZipkinCollectorSyncProcessor<H> {
  pub fn new(handler: H) -> ZipkinCollectorSyncProcessor<H> {
    ZipkinCollectorSyncProcessor {
      handler,
    }
  }
  fn process_submit_zipkin_batch(&self, incoming_sequence_number: i32, i_prot: &mut dyn TInputProtocol, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    TZipkinCollectorProcessFunctions::process_submit_zipkin_batch(&self.handler, incoming_sequence_number, i_prot, o_prot)
  }
}

pub struct TZipkinCollectorProcessFunctions;

impl TZipkinCollectorProcessFunctions {
  pub fn process_submit_zipkin_batch<H: ZipkinCollectorSyncHandler>(handler: &H, incoming_sequence_number: i32, i_prot: &mut dyn TInputProtocol, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let args = ZipkinCollectorSubmitZipkinBatchArgs::read_from_in_protocol(i_prot)?;
    match handler.handle_submit_zipkin_batch(args.spans) {
      Ok(handler_return) => {
        let message_ident = TMessageIdentifier::new("submitZipkinBatch", TMessageType::Reply, incoming_sequence_number);
        o_prot.write_message_begin(&message_ident)?;
        let ret = ZipkinCollectorSubmitZipkinBatchResult { result_value: Some(handler_return) };
        ret.write_to_out_protocol(o_prot)?;
        o_prot.write_message_end()?;
        o_prot.flush()
      },
      Err(e) => {
        match e {
          thrift::Error::Application(app_err) => {
            let message_ident = TMessageIdentifier::new("submitZipkinBatch", TMessageType::Exception, incoming_sequence_number);
            o_prot.write_message_begin(&message_ident)?;
            thrift::Error::write_application_error_to_out_protocol(&app_err, o_prot)?;
            o_prot.write_message_end()?;
            o_prot.flush()
          },
          _ => {
            let ret_err = {
              ApplicationError::new(
                ApplicationErrorKind::Unknown,
                e.to_string()
              )
            };
            let message_ident = TMessageIdentifier::new("submitZipkinBatch", TMessageType::Exception, incoming_sequence_number);
            o_prot.write_message_begin(&message_ident)?;
            thrift::Error::write_application_error_to_out_protocol(&ret_err, o_prot)?;
            o_prot.write_message_end()?;
            o_prot.flush()
          },
        }
      },
    }
  }
}

impl <H: ZipkinCollectorSyncHandler> TProcessor for ZipkinCollectorSyncProcessor<H> {
  fn process(&self, i_prot: &mut dyn TInputProtocol, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let message_ident = i_prot.read_message_begin()?;
    let res = match &*message_ident.name {
      "submitZipkinBatch" => {
        self.process_submit_zipkin_batch(message_ident.sequence_number, i_prot, o_prot)
      },
      method => {
        Err(
          thrift::Error::Application(
            ApplicationError::new(
              ApplicationErrorKind::UnknownMethod,
              format!("unknown method {}", method)
            )
          )
        )
      },
    };
    thrift::server::handle_process_result(&message_ident, res, o_prot)
  }
}

//
// ZipkinCollectorSubmitZipkinBatchArgs
//

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ZipkinCollectorSubmitZipkinBatchArgs {
  spans: Vec<Span>,
}

impl ZipkinCollectorSubmitZipkinBatchArgs {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ZipkinCollectorSubmitZipkinBatchArgs> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<Vec<Span>> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<Span> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_2 = Span::read_from_in_protocol(i_prot)?;
            val.push(list_elem_2);
          }
          i_prot.read_list_end()?;
          f_1 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("ZipkinCollectorSubmitZipkinBatchArgs.spans", &f_1)?;
    let ret = ZipkinCollectorSubmitZipkinBatchArgs {
      spans: f_1.expect("auto-generated code should have checked for presence of required fields"),
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("submitZipkinBatch_args");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("spans", TType::List, 1))?;
    o_prot.write_list_begin(&TListIdentifier::new(TType::Struct, self.spans.len() as i32))?;
    for e in &self.spans {
      e.write_to_out_protocol(o_prot)?;
      o_prot.write_list_end()?;
    }
    o_prot.write_field_end()?;
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

//
// ZipkinCollectorSubmitZipkinBatchResult
//

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ZipkinCollectorSubmitZipkinBatchResult {
  result_value: Option<Vec<Response>>,
}

impl ZipkinCollectorSubmitZipkinBatchResult {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ZipkinCollectorSubmitZipkinBatchResult> {
    i_prot.read_struct_begin()?;
    let mut f_0: Option<Vec<Response>> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        0 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<Response> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_3 = Response::read_from_in_protocol(i_prot)?;
            val.push(list_elem_3);
          }
          i_prot.read_list_end()?;
          f_0 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    let ret = ZipkinCollectorSubmitZipkinBatchResult {
      result_value: f_0,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ZipkinCollectorSubmitZipkinBatchResult");
    o_prot.write_struct_begin(&struct_ident)?;
    if let Some(ref fld_var) = self.result_value {
      o_prot.write_field_begin(&TFieldIdentifier::new("result_value", TType::List, 0))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::Struct, fld_var.len() as i32))?;
      for e in fld_var {
        e.write_to_out_protocol(o_prot)?;
        o_prot.write_list_end()?;
      }
      o_prot.write_field_end()?;
      ()
    } else {
      ()
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
  fn ok_or(self) -> thrift::Result<Vec<Response>> {
    if self.result_value.is_some() {
      Ok(self.result_value.unwrap())
    } else {
      Err(
        thrift::Error::Application(
          ApplicationError::new(
            ApplicationErrorKind::MissingResult,
            "no result received for ZipkinCollectorSubmitZipkinBatch"
          )
        )
      )
    }
  }
}

