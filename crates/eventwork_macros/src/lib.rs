use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(SubscribeById, attributes(subscribe_id))]
pub fn derive_subscribe_by_id(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    
    // Generate the struct names
    let subscribe_struct_name = quote::format_ident!("SubscribeTo{}", name);
    let unsubscribe_struct_name = quote::format_ident!("UnsubscribeFrom{}", name);
    
    // Get the subscribe_id field and its type, if any
    let subscribe_id_field = find_subscribe_id_field(&ast.data);
    
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

    // Add the custom subscription ID field to the struct
    let custom_field = if subscribe_id_field.is_none() {
        quote! {
            // Add a field to the struct to store custom subscription ID
            impl #name {
                #[doc(hidden)]
                #[serde(skip)]
                #[serde(default)]
                _subscription_id: Option<String>,
            }
        }
    } else {
        quote! {}
    };

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

    quote! {
        #custom_field
        
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
