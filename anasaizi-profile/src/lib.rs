use proc_macro::TokenStream;
use quote::quote;
use ::syn::{self,
            ItemFn,
            parse_macro_input,
            parse_quote,
            spanned::Spanned,
};
use syn::{Stmt, Expr};

#[cfg(not(feature = "profile"))]
#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    item.into()
}

#[cfg(feature = "profile")]
#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("{}", item.to_string());
    println!("{}", attr.to_string());

    let mut input = syn::parse_macro_input!(item as syn::ItemFn);

    let fn_name = input.sig.ident.to_string();
    let attributes = attr.to_string();

    let lock_inj = quote::quote! {
        let mut lock = PROFILER.lock().unwrap();
    };

    let profile_inj = quote::quote! {
         let fn_profile = lock.start_profile(format!("fn {}() ({})", #fn_name, #attributes));
    };

    let lock_drop_inj = quote::quote! {
       drop(lock);
    };

    let drop_profile = quote::quote! {
      fn_profile.end_profile();
    };

    input.block.stmts.insert(0, syn::parse(lock_inj.into()).unwrap());
    input.block.stmts.insert(1, syn::parse(profile_inj.into()).unwrap());
    input.block.stmts.insert(2, syn::parse(lock_drop_inj.into()).unwrap());
    input.block.stmts.insert(input.block.stmts.len(), syn::parse(drop_profile.into()).unwrap());

    let output = quote::quote! {
        #input
    };

    output.into()
}