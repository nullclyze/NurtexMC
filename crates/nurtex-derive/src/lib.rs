use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

mod utils;

use utils::*;

/// Макрос автоматической генерации `read` и `write` методов для пакета.
/// 
/// **Доступные атрибуты:**
/// 
/// - `#packet[skip]`: Просто игнорирование поля
/// - `#packet[varint]`: Читает `i32` как `VarInt`
/// - `#packet[varlong]`: Читает `i64` как `VarLong`
/// - `#packet[vec_varint]`: Читает вектор из `i64` как `VarLong`
/// - `#packet[vec_varlong]`: Читает вектор из `i64` как `VarLong`
/// - `#packet[vec_end]`: Читает все оставшиеся байты из пакета (применяется если поле в конце пакета)
/// - `#packet[option]`: Корректно работает с опциональным значением
/// 
/// Атрибуты `#packet[varint]` и `#packet[varlong]` нужны для точного определения типа, так как
/// в пакеты записывается лишь полученное значение после чтения байтов через методы 
/// трейтов `VarInt` / `VarLong`, соответственно макрос сам не может точно определить 
/// что за тип подразумевает поле (`VarInt` или просто `i32`, `VarLong` или просто `i64`)
/// 
/// Так же есть оговорка: Если в `Option<>` лежит `VarInt` / `VarLong` значение, 
/// то указывается атрибут `#packet[varint]` / `#packet[varlong]`, макрос
/// автоматически определит `Option<>` и корректно достанет значение
/// `VarInt` / `VarLong` из него
#[proc_macro_derive(Packet, attributes(packet))]
pub fn derive_packet(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let name = &input.ident;
  let read_impl = generate_read(&input);
  let write_impl = generate_write(&input);

  let expanded = quote! {
    impl #name {
      pub fn read(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
        #read_impl
      }

      pub fn write(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
        #write_impl
        Ok(())
      }
    }
  };

  expanded.into()
}

/// Макрос автоматической генерации методов для союза пакетов
/// и для всех вариантов (то есть пакетов) этого союза
#[proc_macro_derive(PacketUnion, attributes(packet_id))]
pub fn derive_packet_union(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let enum_name = &input.ident;
  
  let variants = match &input.data {
    Data::Enum(data) => &data.variants,
    _ => {
      return syn::Error::new_spanned(enum_name, "PacketUnion can only be applied to enums")
        .to_compile_error()
        .into();
    }
  };

  let enum_impl = quote! {
    impl crate::IntoPacket<#enum_name> for #enum_name {
      fn into_packet(self) -> #enum_name {
        self
      }
    }
  };

  let variant_impls = variants.iter().map(|variant| {
    let variant_name = &variant.ident;

    match &variant.fields {
      syn::Fields::Unnamed(fields) => {
        if fields.unnamed.len() == 1 {
          let field_type = &fields.unnamed[0].ty;
          
          quote! {
            impl crate::IntoPacket<#enum_name> for #field_type {
              fn into_packet(self) -> #enum_name {
                #enum_name::#variant_name(self)
              }
            }
          }
        } else {
          quote! {
            compile_error!("PacketUnion variants must have exactly one field");
          }
        }
      }
      syn::Fields::Named(_) => {
        quote! {
          compile_error!("PacketUnion variants must use unnamed fields");
        }
      }
      syn::Fields::Unit => {
        quote! {
          compile_error!("PacketUnion variants cannot be unit variants");
        }
      }
    }
  });

  let has_packet_ids = variants.iter().any(|v| utils::extract_packet_id(v).is_some());

  let packet_impl = if has_packet_ids {
    let id_arms = variants.iter().filter_map(|variant| {
      let variant_name = &variant.ident;
      utils::extract_packet_id(variant).map(|id| {
        quote! {
          Self::#variant_name(_) => #id,
        }
      })
    });

    let read_arms = variants.iter().filter_map(|variant| {
      let variant_name = &variant.ident;
      
      match &variant.fields {
        syn::Fields::Unnamed(fields) => {
          if fields.unnamed.len() == 1 {
            let field_type = &fields.unnamed[0].ty;
            utils::extract_packet_id(variant).map(|id| {
              quote! {
                #id => Some(Self::#variant_name(<#field_type>::read(buf)?)),
              }
            })
          } else {
            None
          }
        }
        _ => None
      }
    });

    let write_arms = variants.iter().map(|variant| {
      let variant_name = &variant.ident;
      
      quote! {
        Self::#variant_name(p) => p.write(buf),
      }
    });

    quote! {
      impl crate::ProtocolPacket for #enum_name {
        fn id(&self) -> u32 {
          match self {
            #(#id_arms)*
          }
        }

        fn read(id: u32, buf: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
          match id {
            #(#read_arms)*
            _ => None,
          }
        }

        fn write(&self, buf: &mut impl std::io::Write) -> std::io::Result<()> {
          match self {
            #(#write_arms)*
          }
        }
      }
    }
  } else {
    quote! {}
  };

  let expanded = quote! {
    #enum_impl

    #(#variant_impls)*

    #packet_impl
  };

  expanded.into()
}
