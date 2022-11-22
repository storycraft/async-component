use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, IdentFragment};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, ExprPath, Fields, Ident, Index};

#[proc_macro_derive(AsyncComponent, attributes(component, state, stream))]
pub fn component_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    proc_macro::TokenStream::from(impl_component_stream(&input))
}

fn extract_path_attribute(ident: &str, attrs: &[Attribute]) -> Option<Option<ExprPath>> {
    for attr in attrs {
        if !attr.path.is_ident(ident) {
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

    let state_update_call = extract_path_attribute("component", &input.attrs).map(|path| {
        quote! {
            if !result.is_empty() {
                #path(&mut self);
            }
        }
    });

    let poll_next_body = match input.data {
        Data::Struct(ref data) => {
            let state_poll = state_stream_poll_body(&data.fields);
            let stream_poll = sub_stream_stream_poll_body(&data.fields);
            let component_poll = component_stream_poll_body(&data.fields);

            quote! {
                #state_poll

                #state_update_call

                #stream_poll

                #component_poll
            }
        }
        Data::Enum(_) => unimplemented!("Derive cannot be applied to enum"),
        Data::Union(_) => unimplemented!("Derive cannot be applied to union"),
    };

    quote! {
        impl #impl_generics ::async_component::AsyncComponent for #name #ty_generics #where_clause {
            fn poll_next(
                mut self: ::std::pin::Pin<&mut Self>,
                cx: &mut ::std::task::Context<'_>
            ) -> ::std::task::Poll<::async_component::ComponentPollFlags> {
                let mut result = ::async_component::ComponentPollFlags::empty();

                #poll_next_body

                if result.is_empty() {
                    ::std::task::Poll::Pending
                } else {
                    ::std::task::Poll::Ready(result)
                }
            }
        }
    }
}

fn state_stream_poll_body(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            let iter = fields.named.iter().filter_map(|field| {
                let method_name = extract_path_attribute("state", &field.attrs)?;
                let name = field.ident.as_ref().unwrap();

                Some(field_state_stream_poll_body(name, method_name))
            });

            quote! {
                #(#iter)*
            }
        }
        Fields::Unnamed(fields) => {
            let iter = fields
                .unnamed
                .iter()
                .enumerate()
                .filter_map(|(index, field)| {
                    let method_name = extract_path_attribute("state", &field.attrs)?;
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
    }
}

fn field_state_stream_poll_body(
    name: impl IdentFragment,
    method_name: Option<ExprPath>,
) -> TokenStream {
    let name = format_ident!("{}", name);

    let method_call = method_name.map(|path| quote! { #path(&mut self); });

    quote_spanned! { name.span() =>
        if ::async_component::StateCell::refresh(
            &mut self.#name
        ) {
            #method_call
            result |= ::async_component::ComponentPollFlags::STATE;
        }
    }
}

fn component_stream_poll_body(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            let iter = fields.named.iter().filter_map(|field| {
                let method_name = extract_path_attribute("component", &field.attrs)?;
                let name = field.ident.as_ref().unwrap();

                Some(field_component_stream_poll_body(name, method_name))
            });

            quote! {
                #(#iter)*
            }
        }
        Fields::Unnamed(fields) => {
            let iter = fields
                .unnamed
                .iter()
                .enumerate()
                .filter_map(|(index, field)| {
                    let method_name = extract_path_attribute("component", &field.attrs)?;
                    let index = Index::from(index);

                    Some(field_component_stream_poll_body(index, method_name))
                });

            quote! {
                #(#iter)*
            }
        }
        Fields::Unit => {
            quote!()
        }
    }
}

fn field_component_stream_poll_body(
    name: impl IdentFragment,
    method_name: Option<ExprPath>,
) -> TokenStream {
    let name = format_ident!("{}", name);

    let method_call = method_name.map(|path| quote! { #path(&mut self, recv); });

    quote_spanned! { name.span() =>
        if let ::std::task::Poll::Ready(recv)
            = ::async_component::AsyncComponent::poll_next(::std::pin::Pin::new(&mut self.#name), cx) {
            #method_call
            result |= recv;
        }
    }
}

fn sub_stream_stream_poll_body(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            let iter = fields.named.iter().filter_map(|field| {
                let method_name = extract_path_attribute("stream", &field.attrs)?;
                let name = field.ident.as_ref().unwrap();

                Some(field_sub_stream_stream_poll_body(name, method_name))
            });

            quote! {
                #(#iter)*
            }
        }
        Fields::Unnamed(fields) => {
            let iter = fields
                .unnamed
                .iter()
                .enumerate()
                .filter_map(|(index, field)| {
                    let method_name = extract_path_attribute("stream", &field.attrs)?;
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
        while let ::std::task::Poll::Ready(Some(#recv_item))
            = ::async_component::__private::Stream::poll_next(::std::pin::Pin::new(&mut self.#name), cx) {
            #method_call
            result |= ::async_component::ComponentPollFlags::STREAM;
        }
    }
}
