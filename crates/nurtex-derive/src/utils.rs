use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, GenericArgument, PathArguments};

/// Функция получения атрибута пакета
pub fn get_packet_attr(f: &syn::Field) -> Option<String> {
  f.attrs
    .iter()
    .find(|a| a.path().is_ident("packet"))
    .and_then(|a| a.parse_args::<syn::Ident>().ok())
    .map(|i| i.to_string())
}

/// Функция извлечения типа из `Option<T>`
pub fn extract_option_inner_type(ty: &Type) -> Option<Type> {
  if let Type::Path(type_path) = ty {
    if let Some(segment) = type_path.path.segments.last() {
      if segment.ident == "Option" {
        if let PathArguments::AngleBracketed(args) = &segment.arguments {
          if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
            return Some(inner_ty.clone());
          }
        }
      }
    }
  }
  None
}

/// Функция проверки, является ли тип Option<T>
pub fn is_option_type(ty: &Type) -> bool {
  extract_option_inner_type(ty).is_some()
}

/// Функция генерации кода для чтения значения с учётом атрибута
fn generate_read_value(ty: &Type, attr: Option<&str>) -> proc_macro2::TokenStream {
  match attr {
    Some("varint") => quote! { <i32 as nurtex_codec::VarInt>::read_varint(buffer)? },
    Some("varlong") => quote! { <i64 as nurtex_codec::VarLong>::read_varlong(buffer)? },
    Some("vec_end") => quote! {
      {
        let remaining = buffer.get_ref().len() - buffer.position() as usize;
        let mut vec = vec![0u8; remaining];

        for byte in &mut vec {
          *byte = u8::read_buf(buffer)?;
        }

        vec
      }
    },
    Some("vec_varint") => quote! {
      {
        let count = <i32 as nurtex_codec::VarInt>::read_varint(buffer)? as usize;
        let mut vec = Vec::with_capacity(count);

        for _ in 0..count {
          vec.push(<i32 as nurtex_codec::VarInt>::read_varint(buffer)?);
        }

        vec
      }
    },
    Some("vec_varlong") => quote! {
      {
        let count = <i32 as nurtex_codec::VarInt>::read_varint(buffer)? as usize;
        let mut vec = Vec::with_capacity(count);

        for _ in 0..count {
          vec.push(<i64 as nurtex_codec::VarLong>::read_varlong(buffer)?);
        }

        vec
      }
    },
    _ => quote! { <#ty as nurtex_codec::Buffer>::read_buf(buffer)? },
  }
}

/// Функция генерации кода для записи значения с учётом атрибута
fn generate_write_value(value: proc_macro2::TokenStream, _ty: &Type, attr: Option<&str>) -> proc_macro2::TokenStream {
  match attr {
    Some("varint") => quote! { <i32 as nurtex_codec::VarInt>::write_varint(&#value, buffer)?; },
    Some("varlong") => quote! { <i64 as nurtex_codec::VarLong>::write_varlong(&#value, buffer)?; },
    Some("vec_end") => quote! {
      <i32 as nurtex_codec::VarInt>::write_varint(&(#value.len() as i32), buffer)?;
      for byte in &#value {
        byte.write_buf(buffer)?;
      }
    },
    Some("vec_varint") => quote! {
      <i32 as nurtex_codec::VarInt>::write_varint(&(#value.len() as i32), buffer)?;
      for item in &#value {
        <i32 as nurtex_codec::VarInt>::write_varint(item, buffer)?;
      }
    },
    Some("vec_varlong") => quote! {
      <i32 as nurtex_codec::VarInt>::write_varint(&(#value.len() as i32), buffer)?;
      for item in &#value {
        <i64 as nurtex_codec::VarLong>::write_varlong(item, buffer)?;
      }
    },
    _ => quote! { #value.write_buf(buffer)?; },
  }
}

/// Функция генерации чтения пакета
pub fn generate_read(input: &DeriveInput) -> proc_macro2::TokenStream {
  match &input.data {
    Data::Struct(data) => match &data.fields {
      Fields::Named(fields) => {
        let field_reads = fields.named.iter().map(|f| {
          let name = &f.ident;
          let ty = &f.ty;

          let attr = get_packet_attr(f);

          match attr.as_deref() {
            Some("skip") => quote! {},
            Some("option") => {
              if let Some(inner_ty) = extract_option_inner_type(ty) {
                quote! {
                  #name: if <bool as nurtex_codec::Buffer>::read_buf(buffer)? {
                    Some(<#inner_ty as nurtex_codec::Buffer>::read_buf(buffer)?)
                  } else {
                    None
                  }
                }
              } else {
                quote! { compile_error!("Option attribute requires Option<T> type") }
              }
            },
            _ => {
              if is_option_type(ty) {
                if let Some(inner_ty) = extract_option_inner_type(ty) {
                  let read_value = generate_read_value(&inner_ty, attr.as_deref());
                  quote! {
                    #name: if <bool as nurtex_codec::Buffer>::read_buf(buffer)? {
                      Some(#read_value)
                    } else {
                      None
                    }
                  }
                } else {
                  quote! { compile_error!("Failed to extract Option inner type") }
                }
              } else {
                let read_value = generate_read_value(ty, attr.as_deref());
                quote! { #name: #read_value }
              }
            }
          }
        });

        quote! {
          Some(Self {
            #(#field_reads),*
          })
        }
      }
      Fields::Unit => quote! { Some(Self) },
      _ => quote! { compile_error!("Packet derive only supports named fields") },
    },
    _ => quote! { compile_error!("Packet derive only supports structs") },
  }
}

/// Функция генерации записи пакета
pub fn generate_write(input: &DeriveInput) -> proc_macro2::TokenStream {
  match &input.data {
    Data::Struct(data) => match &data.fields {
      Fields::Named(fields) => {
        let field_writes = fields.named.iter().map(|f| {
          let name = &f.ident;
          let ty = &f.ty;
          let attr = get_packet_attr(f);

          match attr.as_deref() {
            Some("skip") => quote! {},
            Some("option") => {
              if let Some(inner_ty) = extract_option_inner_type(ty) {
                quote! {
                  <bool as nurtex_codec::Buffer>::write_buf(&self.#name.is_some(), buffer)?;
                  if let Some(val) = &self.#name {
                    <#inner_ty as nurtex_codec::Buffer>::write_buf(val, buffer)?;
                  }
                }
              } else {
                quote! { compile_error!("Option attribute requires Option<T> type") }
              }
            },
            _ => {
              if is_option_type(ty) {
                if let Some(inner_ty) = extract_option_inner_type(ty) {
                  let write_value = generate_write_value(quote! { val }, &inner_ty, attr.as_deref());
                  quote! {
                    <bool as nurtex_codec::Buffer>::write_buf(&self.#name.is_some(), buffer)?;
                    if let Some(val) = &self.#name {
                      #write_value
                    }
                  }
                } else {
                  quote! { compile_error!("Failed to extract Option inner type") }
                }
              } else {
                let write_value = generate_write_value(quote! { self.#name }, ty, attr.as_deref());
                quote! { #write_value }
              }
            }
          }
        });

        quote! {
          #(#field_writes)*
        }
      }
      Fields::Unit => quote! {},
      _ => quote! { compile_error!("Packet derive only supports named fields") },
    },
    _ => quote! { compile_error!("Packet derive only supports structs") },
  }
}

/// Функция извлечения ID пакета
pub fn extract_packet_id(variant: &syn::Variant) -> Option<u32> {
  variant.attrs.iter()
    .find(|a| a.path().is_ident("packet_id"))
    .and_then(|a| {
      if let syn::Meta::NameValue(nv) = &a.meta {
        if let syn::Expr::Lit(expr_lit) = &nv.value {
          match &expr_lit.lit {
            syn::Lit::Int(lit_int) => {
              let s = lit_int.to_string();
              if s.starts_with("0x") {
                u32::from_str_radix(&s[2..], 16).ok()
              } else {
                lit_int.base10_parse::<u32>().ok()
              }
            }
            syn::Lit::Str(lit_str) => {
              let s = lit_str.value();
              if s.starts_with("0x") {
                u32::from_str_radix(&s[2..], 16).ok()
              } else {
                s.parse::<u32>().ok()
              }
            }
            _ => None
          }
        } else {
          None
        }
      } else {
        None
      }
    })
}