use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input,
    parse_quote,
    punctuated::Punctuated,
    token::Comma,
    AttributeArgs,
    Block,
    FnArg,
    ImplItem,
    ImplItemMethod,
    Item,
    ReturnType,
    Stmt,
};

macro_rules! parse_system_input {
    ($i:ident) => {
        if let ImplItem::Method(input) = parse_macro_input!($i as ImplItem) {
            input
        } else {
            panic!("this attribute macro only works on trait methods")
        }
    };
}

fn impl_system<F>(input: ImplItemMethod, output: ReturnType, body: F) -> TokenStream
where
    F: FnOnce(&Punctuated<FnArg, Comma>, &Block) -> proc_macro2::TokenStream,
{
    let args = &input.sig.inputs;
    let block = &input.block;

    let body = if let Some(Stmt::Item(Item::Verbatim(item))) = block.stmts.get(0) {
        item.clone()
    } else {
        let body = body(args, block);
        quote! { { #body } }
    };

    let mut sig = input.sig;

    sig.inputs.clear();
    sig.output = output;

    quote! {
        #sig #body
    }
    .into()
}

#[proc_macro_attribute]
pub fn system(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_system_input!(input);

    impl_system(
        input,
        parse_quote! { -> bevy::ecs::system::BoxedSystem },
        |args, block| {
            quote! { Box::new(bevy::ecs::system::IntoSystem::into_system(|#args| #block)) }
        },
    )
}

#[proc_macro_attribute]
pub fn system_config(args: TokenStream, input: TokenStream) -> TokenStream {
    let _attr_args = parse_macro_input!(args as AttributeArgs);
    let input = parse_system_input!(input);

    impl_system(
        input,
        parse_quote! { -> bevy::ecs::schedule::SystemConfig },
        |args, block| {
            quote! { (|#args| #block).into_config() }
        },
    )
}

#[proc_macro_attribute]
pub fn system_app_config(args: TokenStream, input: TokenStream) -> TokenStream {
    let _attr_args = parse_macro_input!(args as AttributeArgs);
    let input = parse_system_input!(input);

    impl_system(
        input,
        parse_quote! { -> bevy::app::SystemAppConfig },
        |args, block| {
            quote! { (|#args| #block).into_app_config() }
        },
    )
}
