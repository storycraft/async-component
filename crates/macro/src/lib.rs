use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, IdentFragment};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, ExprPath, Fields, Index};

#[proc_macro_derive(AsyncComponent, attributes(component, state))]
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

    let update_component_body = match input.data {
        Data::Struct(ref data) => {
            let state_update_call = extract_path_attribute("component", &input.attrs).map(|path| {
                quote! {
                    if updated {
                        #path(self);
                    }
                }
            });

            let state_poll = update_state_body(&data.fields);
            let component_poll = component_update_body(&data.fields);

            quote! {
                let mut updated = false;

                #component_poll

                #state_poll

                #state_update_call

                updated
            }
        }
        Data::Enum(_) => unimplemented!("Derive cannot be applied to enum"),
        Data::Union(_) => unimplemented!("Derive cannot be applied to union"),
    };

    quote! {
        impl #impl_generics ::async_component::AsyncComponent for #name #ty_generics #where_clause {
            fn update_component(&mut self) -> bool {
                #update_component_body
            }
        }
    }
}

fn update_state_body(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            let iter = fields.named.iter().filter_map(|field| {
                let method_name = extract_path_attribute("state", &field.attrs)?;
                let name = field.ident.as_ref().unwrap();

                Some(field_state_update_body(name, method_name))
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

                    Some(field_state_update_body(index, method_name))
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

fn field_state_update_body(name: impl IdentFragment, method_name: Option<ExprPath>) -> TokenStream {
    let name = format_ident!("{}", name);

    let method_call = method_name.map(|path| quote! { #path(self, _recv); });

    let update = update_result();
    quote_spanned! { name.span() =>
        if let Some(_recv) = ::async_component::State::update(&mut self.#name) {
            #method_call
            #update
        }
    }
}

fn component_update_body(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            let iter = fields.named.iter().filter_map(|field| {
                let method_name = extract_path_attribute("component", &field.attrs)?;
                let name = field.ident.as_ref().unwrap();

                Some(field_component_update_body(name, method_name))
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

                    Some(field_component_update_body(index, method_name))
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

fn field_component_update_body(
    name: impl IdentFragment,
    method_name: Option<ExprPath>,
) -> TokenStream {
    let name = format_ident!("{}", name);

    let method_call = method_name.map(|path| quote! { #path(self); });

    let update = update_result();
    quote_spanned! { name.span() =>
        if ::async_component::AsyncComponent::update_component(&mut self.#name) {
            #method_call
            #update
        }
    }
}
fn update_result() -> TokenStream {
    quote! {
        if !updated {
            updated = true
        }
    }
}
