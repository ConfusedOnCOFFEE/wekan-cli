use proc_macro2::{self};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(UnauthenticationClient)]
pub fn derive_unauthorized_client(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let output: proc_macro2::TokenStream = quote! {
        impl crate::http::authentication::Unauthorized for #name {}
        impl AddressConfig for LoginClient {
            fn get_address(&self) -> String {
                self.config.get_address()
            }

            fn get_api_address(&self) -> String {
                self.get_address() + "/api/"
            }
        }

    };
    proc_macro::TokenStream::from(output)
}

#[proc_macro_derive(TokenManagerClient)]
pub fn derive_authentication_client(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let output: proc_macro2::TokenStream = quote! {

        impl crate::http::authentication::Login for #name {}
        impl crate::http::authentication::TokenManager for #name {}
        impl crate::http::preflight_request::Client for #name {}

        impl crate::config::AddressConfig for #name {
            fn get_address(&self) -> String {
                self.config.get_address()
            }

            fn get_api_address(&self) -> String {
                self.get_address() + "/api/"
            }
        }
        #[async_trait]
        impl wekan_common::validation::authentication::StoreToken for #name {
            async fn store_token(&mut self, t: Token) -> Token {
                self.config.store_token(t).await
            }
        }

    };
    proc_macro::TokenStream::from(output)
}

#[proc_macro_derive(ArtifactClient)]
pub fn derive_artifact_client(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let _attrs = input.attrs;
    let generics = input.generics;
    let (_impl_generics, _ty_generics, _where_clause) = generics.split_for_impl();

    let output: proc_macro2::TokenStream = quote! {
        impl crate::http::authentication::TokenManager for #name {}
        impl crate::http::authentication::Login for #name {}
        impl crate::http::client::HttpClient for #name {}
        impl crate::http::operation::Artifacts for #name {}
        impl crate::http::operation::Operation for #name {}
        impl crate::config::ArtifactApi for  #name {
            fn get_artifacts_url(&self) -> String {
                self.config.get_api_address() + &self.base
            }
            fn get_artifact_url(&self, id: &str) -> String {
                self.config.get_api_address() + &self.base + id
            }
        }
        impl crate::config::AddressConfig for #name {
            fn get_address(&self) -> String {
                self.config.get_address()
            }

            fn get_api_address(&self) -> String {
                self.get_address() + "/api/"
            }
        }
        #[async_trait]
        impl wekan_common::validation::authentication::StoreToken for #name {
            async fn store_token(&mut self, t: Token) -> Token {
                self.config.store_token(t).await
            }
        }
    };
    // panic!("{:?}", proc_macro2::TokenStream::to_string(&output));
    proc_macro::TokenStream::from(output)
}

#[proc_macro_derive(TokenConfig)]
pub fn derive_token_config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let output: proc_macro2::TokenStream = quote! {
        #[cfg(not(test))]
        impl wekan_common::validation::authentication::TokenHeader for #name {
            fn get_usertoken(&self) -> Token {
                match &self.config.usertoken {
                    Some(t) => t.to_owned(),
                    None => panic!("No token")
                }
            }

            fn get_token(&self) -> String {
                *self.get_usertoken().token
            }
            fn set_token(&mut self, t: Token) -> Token {
                self.config.usertoken = Some(t.to_owned());
                t
            }

            fn get_user_id(&self) -> String {
                *self.get_usertoken().id
            }
        }
        #[cfg(test)]
        impl wekan_common::validation::authentication::TokenHeader for #name {
            fn get_usertoken(&self) -> Token {
                Token {
                    id: Box::new(String::from("B8D3e2qeXitTeqm9s")),
                    token: Box::new(String::from("yNa1VR1Cz6nTzNirWPm2dRNYjdu-EM6LxKDIT0pIYsi")),
                    token_expires: Box::new(String::from("2022-08-30T19:37:47.170Z")),
                }
            }

            fn get_token(&self) -> String {
                *self.get_usertoken().token
            }
            fn set_token(&mut self, t: Token) -> Token {
                self.config.usertoken = Some(t.to_owned());
                t
            }

            fn get_user_id(&self) -> String {
                *self.get_usertoken().id
            }
        }

    };
    proc_macro::TokenStream::from(output)
}
