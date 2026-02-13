use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(UiDebug)]
pub fn derive_ui_debug(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl crate::ui_components::UIComponent for #name {
            fn name(&self) -> &str {
                stringify!(#name)
            }

            fn render_test(&self) {
                println!();
                println!("--- DEBUG UI COMPONENT: {} ---", self.name());
                println!("State: {:?}", self);
                println!("--------------------------------------");
                println!();

                // Call the component's own test logic if it exists
                // We assume there's a method named `test_output` for custom rendering
                self.test_output();
            }
        }
    };

    TokenStream::from(expanded)
}
