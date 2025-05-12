use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro]
pub fn notify_component_subscribers(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ExprTuple);
    
    // Extract the component type and type name from the tuple
    if input.elems.len() != 2 {
        return syn::Error::new_spanned(
            input,
            "Expected exactly two arguments: component type and type name string",
        )
        .to_compile_error()
        .into();
    }
    
    let component_type = &input.elems[0];
    let type_name = &input.elems[1];
    
    let expanded = quote! {
        pub fn notify_component_subscribers(
            changed_components: Query<(Entity, &#component_type), Changed<#component_type>>,
            subscriptions: Res<ComponentSubscriptions>,
            mut outbound: EventWriter<OutboundMessage<ReflectedEntityData>>,
        ) {
            for (entity, component) in changed_components.iter() {
                let entity_id = entity.to_bits();
                
                // Get subscribers for this entity and component type
                let subscribers = subscriptions.get_subscribers(entity_id, #type_name);
                
                if !subscribers.is_empty() {
                    // Serialize the component
                    if let Ok(component_data) = bincode::serialize(component) {
                        let reflected_data = ReflectedEntityData {
                            entity_id,
                            component_type: #type_name.to_string(),
                            data: component_data,
                        };
                        
                        // Send to all subscribers
                        for (client_id, _field_path) in subscribers {
                            // TODO: Filter by field path if specified
                            outbound.send(OutboundMessage::new(ReflectedEntityData::NAME.to_string(), reflected_data.clone()).for_client(client_id));
                        }
                    }
                }
            }
        }
    };
    
    expanded.into()
}

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
            pub struct #subscribe_struct_name;
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
            pub struct #unsubscribe_struct_name;
        }
    };

    // Implement NetworkMessage for both structs
    let subscribe_name = format!("{}:Subscribe", name);
    let unsubscribe_name = format!("{}:Unsubscribe", name);

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
            }
        },
        None => quote! {
            impl SubscriptionMessage for #name {
                type SubscribeRequest = #subscribe_struct_name;
                type UnsubscribeRequest = #unsubscribe_struct_name;
                type SubscriptionParams = String;

                fn get_subscription_params(&self) -> Self::SubscriptionParams {
                    Self::NAME.to_string()
                }

                fn create_subscription_request(_params: Self::SubscriptionParams) -> Self::SubscribeRequest {
                    #subscribe_struct_name
                }

                fn create_unsubscribe_request(_params: Self::SubscriptionParams) -> Self::UnsubscribeRequest {
                    #unsubscribe_struct_name
                }
            }
        }
    };

    quote! {
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

