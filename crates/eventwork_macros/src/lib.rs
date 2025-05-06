use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_attribute]
pub fn subscribe_by_id(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input struct
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    
    // Check if there are any attributes passed to the macro
    let _attr_args = parse_macro_input!(attr as syn::AttributeArgs);
    
    // Generate the struct names for subscription messages
    let subscribe_struct_name = quote::format_ident!("SubscribeTo{}", name);
    let unsubscribe_struct_name = quote::format_ident!("UnsubscribeFrom{}", name);
    
    // Find if there's an explicit subscribe_id field
    let subscribe_id_field = find_subscribe_id_field(&input.data);
    
    // Create a modified struct with an added field if needed
    let modified_struct = if subscribe_id_field.is_none() {
        // We need to add a field for the subscription ID
        match &input.data {
            Data::Struct(data_struct) => {
                let mut modified_data_struct = data_struct.clone();
                
                // Add the _subscription_id field
                match &mut modified_data_struct.fields {
                    Fields::Named(fields) => {
                        fields.named.push(
                            syn::Field::parse_named
                                .parse2(quote! {
                                    #[doc(hidden)]
                                    #[serde(skip)]
                                    #[serde(default)]
                                    pub _subscription_id: Option<String>
                                })
                                .unwrap()
                        );
                    }
                    _ => {
                        // For tuple structs or unit structs, we can't easily add fields
                        // Return an error
                        return syn::Error::new_spanned(
                            input,
                            "subscribe_by_id can only be used with structs that have named fields",
                        )
                        .to_compile_error()
                        .into();
                    }
                }
                
                // Create a new struct with the modified fields
                let vis = &input.vis;
                let attrs = &input.attrs;
                let generics = &input.generics;
                
                quote! {
                    #(#attrs)*
                    #vis struct #name #generics {
                        #modified_data_struct
                    }
                }
            }
            _ => {
                // Return an error for non-struct types
                return syn::Error::new_spanned(
                    input,
                    "subscribe_by_id can only be used with structs",
                )
                .to_compile_error()
                .into();
            }
        }
    } else {
        // No need to modify the struct, just pass it through
        quote! { #input }
    };
    
    // Generate the Subscribe and Unsubscribe message structs
    let subscribe_struct = match &subscribe_id_field {
        Some((field_name, _field_type)) => quote! {
            #[derive(Serialize, Deserialize, Debug)]
            pub struct #subscribe_struct_name {
                pub #field_name: String,
            }
        },
        None => quote! {
            #[derive(Serialize, Deserialize, Debug)]
            pub struct #subscribe_struct_name {
                pub subscription_id: String,
            }
        }
    };

    let unsubscribe_struct = match &subscribe_id_field {
        Some((field_name, _field_type)) => quote! {
            #[derive(Serialize, Deserialize, Debug)]
            pub struct #unsubscribe_struct_name {
                pub #field_name: String,
            }
        },
        None => quote! {
            #[derive(Serialize, Deserialize, Debug)]
            pub struct #unsubscribe_struct_name {
                pub subscription_id: String,
            }
        }
    };

    // Implement NetworkMessage for both structs
    let subscribe_name = format!("{}:Subscribe", name);
    let unsubscribe_name = format!("{}:Unsubscribe", name);

    // Implement SubscriptionMessage
    let subscription_impl = match &subscribe_id_field {
        Some((field_name, _field_type)) => quote! {
            impl SubscriptionMessage for #name {
                type SubscribeRequest = #subscribe_struct_name;
                type UnsubscribeRequest = #unsubscribe_struct_name;
                type SubscriptionParams = String;

                fn get_subscription_params(&self) -> Self::SubscriptionParams {
                    self.#field_name.to_string()
                }

                fn create_subscription_request(params: Self::SubscriptionParams) -> Self::SubscribeRequest {
                    #subscribe_struct_name { #field_name: params }
                }

                fn create_unsubscribe_request(params: Self::SubscriptionParams) -> Self::UnsubscribeRequest {
                    #unsubscribe_struct_name { #field_name: params }
                }
                
                // Override the with_subscription_id method - for structs with a subscribe_id field,
                // this is a no-op as the field value is used instead
                fn with_subscription_id(self, _id: impl Into<String>) -> Self {
                    // For types with an explicit subscribe_id field, we don't modify anything
                    self
                }
            }
        },
        None => quote! {
            impl #name {
                // Helper method to get the current subscription ID or default
                fn _get_subscription_id(&self) -> String {
                    self._subscription_id.clone().unwrap_or_else(|| Self::NAME.to_string())
                }
            }

            impl SubscriptionMessage for #name {
                type SubscribeRequest = #subscribe_struct_name;
                type UnsubscribeRequest = #unsubscribe_struct_name;
                type SubscriptionParams = String;

                fn get_subscription_params(&self) -> Self::SubscriptionParams {
                    self._get_subscription_id()
                }

                fn create_subscription_request(params: Self::SubscriptionParams) -> Self::SubscribeRequest {
                    #subscribe_struct_name { subscription_id: params }
                }

                fn create_unsubscribe_request(params: Self::SubscriptionParams) -> Self::UnsubscribeRequest {
                    #unsubscribe_struct_name { subscription_id: params }
                }
                
                // Implement the with_subscription_id method to set a custom ID
                fn with_subscription_id(mut self, id: impl Into<String>) -> Self {
                    self._subscription_id = Some(id.into());
                    self
                }
            }
        }
    };

    // Combine everything
    quote! {
        #modified_struct
        
        #subscribe_struct
        #unsubscribe_struct

        impl NetworkMessage for #subscribe_struct_name {
            const NAME: &'static str = #subscribe_name;
        }

        impl NetworkMessage for #unsubscribe_struct_name {
            const NAME: &'static str = #unsubscribe_name;
        }

        #subscription_impl
    }.into()
}

fn find_subscribe_id_field(data: &Data) -> Option<(syn::Ident, syn::Type)> {
    match data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    fields.named.iter().find(|field| {
                        field.attrs.iter().any(|attr| attr.path().is_ident("subscribe_id"))
                    }).map(|field| (field.ident.clone().unwrap(), field.ty.clone()))
                }
                _ => None,
            }
        }
        _ => None,
    }
}
