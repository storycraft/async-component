use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, IdentFragment};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, ExprPath, Field, Fields, Ident, Index};

#[proc_macro_derive(Component, attributes(component, state, stream))]
pub fn component_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    proc_macro::TokenStream::from(impl_component_stream(&input))
}

fn extract_component_attribute(attrs: &[Attribute]) -> Option<Option<ExprPath>> {
    for attr in attrs {
        if !attr.path.is_ident("component") {
            continue;
        }

        let method_path = attr.parse_args::<ExprPath>().ok();

        return Some(method_path);
    }

    None
}

fn impl_component_stream(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let state_poll = state_stream_poll_body(&input.data);

    let stream_poll = sub_stream_stream_poll_body(&input.data);

    let state_update_call = extract_component_attribute(&input.attrs).map(|path| {
        quote! {
            if result.is_ready() {
                #path(&mut self);
            }
        }
    });

    quote! {
        impl #impl_generics ::futures::Stream for #name #ty_generics #where_clause {
            type Item = ();

            fn poll_next(
                mut self: ::std::pin::Pin<&mut Self>,
                cx: &mut ::std::task::Context<'_>
            ) -> ::std::task::Poll<Option<Self::Item>> {
                let mut result = ::std::task::Poll::Pending;

                #state_poll

                #state_update_call

                #stream_poll

                result
            }
        }
    }
}

fn extract_state_attribute(attrs: &[Attribute]) -> Option<Option<ExprPath>> {
    for attr in attrs {
        if !attr.path.is_ident("state") {
            continue;
        }

        let method_path = attr.parse_args::<ExprPath>().ok();

        return Some(method_path);
    }

    None
}

fn state_stream_poll_body(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let iter = fields.named.iter().filter_map(|field| {
                    let method_name = extract_state_attribute(&field.attrs)?;
                    let name = field.ident.as_ref().unwrap();

                    Some(field_state_stream_poll_body(name, method_name))
                });

                quote! {
                    #(#iter)*
                }
            }
            Fields::Unnamed(ref fields) => {
                let iter = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .filter_map(|(index, field)| {
                        let method_name = extract_state_attribute(&field.attrs)?;
                        let index = Index::from(index);

                        Some(field_state_stream_poll_body(index, method_name))
                    });

                quote! {
                    #(#iter)*
                }
            }
            Fields::Unit => {
                quote!()
            }
        },

        Data::Enum(_) => todo!(),
        Data::Union(_) => todo!(),
    }
}

fn field_state_stream_poll_body(
    name: impl IdentFragment,
    method_name: Option<ExprPath>,
) -> TokenStream {
    let name = format_ident!("{}", name);

    let method_call = method_name.map(|path| quote! { #path(&mut self); });

    quote_spanned! { name.span() =>
        if ::async_component::StateCell::poll_changed(
            ::std::pin::Pin::new(&mut self.#name), cx).is_ready() {
            #method_call
            result = ::std::task::Poll::Ready(Some(()));
        }
    }
}

fn extract_stream_attribute(attrs: &[Attribute]) -> Option<Option<ExprPath>> {
    for attr in attrs {
        if !attr.path.is_ident("stream") {
            continue;
        }

        let method_path = attr.parse_args::<ExprPath>().ok();
        return Some(method_path);
    }

    None
}

fn sub_stream_stream_poll_body(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let iter = fields.named.iter().filter_map(|field| {
                    let method_name = extract_stream_attribute(&field.attrs)?;
                    let name = field.ident.as_ref().unwrap();

                    Some(field_sub_stream_stream_poll_body(name, method_name))
                });

                quote! {
                    #(#iter)*
                }
            }
            Fields::Unnamed(ref fields) => {
                let iter = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .filter_map(|(index, field)| {
                        let method_name = extract_stream_attribute(&field.attrs)?;
                        let index = Index::from(index);

                        Some(field_sub_stream_stream_poll_body(index, method_name))
                    });

                quote! {
                    #(#iter)*
                }
            }
            Fields::Unit => {
                quote!()
            }
        },

        Data::Enum(_) => todo!(),
        Data::Union(_) => todo!(),
    }
}

fn field_sub_stream_stream_poll_body(
    name: impl IdentFragment,
    method_name: Option<ExprPath>,
) -> TokenStream {
    let name = format_ident!("{}", name);
    let recv_item = if method_name.is_none() {
        Ident::new("_", Span::call_site())
    } else {
        Ident::new("recv", Span::call_site())
    };

    let method_call = method_name.map(|path| quote! { #path(&mut self, #recv_item); });

    quote_spanned! { name.span() =>
        if let ::std::task::Poll::Ready(Some(#recv_item))
            = ::futures::Stream::poll_next(::std::pin::Pin::new(&mut self.#name), cx) {
            #method_call
            result = ::std::task::Poll::Ready(Some(()));
        }
    }
}
