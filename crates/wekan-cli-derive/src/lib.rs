use proc_macro2::{self};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(WekanArgs)]
pub fn derive_wekan_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let output: proc_macro2::TokenStream = quote! {
        impl ArtifactName for #name {
            fn get_name(&self) -> Result<String, Error> {
                match &self.name {
                    Some(n) => Ok(n.to_string()),
                    None => Err(CliError::new_msg("Name option '-n' needs to be supplied").as_enum()),
                }
            }
        }
        impl ArgumentRequester<Command> for #name {
            type Args = Self;
            fn get_argument(&self) -> Self {
                self.to_owned()
            }
        }
        impl SubCommandValidator<Command> for #name {
            fn get_command(&self) -> Option<Command> {
                self.command.to_owned()
            }
        }

    };

    // panic!("{:?}", proc_macro2::TokenStream::to_string(&output));
    proc_macro::TokenStream::from(output)
}

#[proc_macro_derive(CommonSubcommands)]
pub fn derive_common_subcommands(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let output: proc_macro2::TokenStream = quote! {
        impl CommonCommandRequester<Command> for #name {
            fn get_common_command(&self) -> Option<Command> {
                self.command.to_owned()
            }
        }
    };
    proc_macro::TokenStream::from(output)
}

#[proc_macro_derive(FulfilmentRunner)]
pub fn derive_fulfillment_runner(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let output: proc_macro2::TokenStream = quote! {
        impl<'a> Fulfillment<'a> for #name<'a> {
           fn get_client(&mut self) -> Client {
               self.client.to_owned()
           }
           fn get_global_options(&mut self) -> &'a RArgs {
               self.global_options
           }
           fn get_display(&mut self) -> CliDisplay {
               self.display.to_owned()
           }

           fn get_format(&mut self) -> &str {
               &self.format
           }
        }

        impl<'a> ArgumentRequester<Command> for #name<'a> {
            type Args = Args;
            fn get_argument(&self) -> Self::Args {
                self.args.get_argument()
            }
        }
    };
    proc_macro::TokenStream::from(output)
}
