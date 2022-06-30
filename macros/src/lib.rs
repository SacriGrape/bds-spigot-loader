use quote::{quote};
use syn::{parse_macro_input, Token, Ident, Type, parenthesized};
use syn::parse::{Parse, ParseStream};

struct HookInput {
    class: Ident,
    method: Ident,
    params: Vec<Type>,
    return_type: Type,
    detour_func: Ident
}

impl Parse for HookInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let class = input.parse()?;
        input.parse::<Token![::]>()?;
        let method = input.parse()?;

        // Reading stuff inside of (param, param, param...)
        let content;
        parenthesized!(content in input);
        let params = content.parse_terminated::<Type, Token![,]>(Type::parse)?.into_iter().collect();

        input.parse::<Token![->]>()?;
        let return_type = input.parse()?;
        input.parse::<Token![=>]>()?;
        let detour_func = input.parse()?;
        Ok(HookInput {
            class,
            method,
            params,
            return_type,
            detour_func,
        })
    }
}

#[proc_macro]
pub fn initialize_hook(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let HookInput {
        class,
        method,
        params,
        return_type,
        detour_func
    } = parse_macro_input!(input as HookInput);

    let clean_name = format!("{}::{}", class, method);

    let tokens = quote! {
        let func = crate::bds_func!(#class::#method(#(#params),*) -> #return_type);

        let hook = unsafe { RawDetour::new(func as *const (), crate::bds::hook::bds_detour::#detour_func as *const ()).expect("Failed to create hook") };
        let hook = unsafe { ManuallyDrop::new(hook) };
        unsafe { hook.enable().expect("Failed to enable hook"); }

        let mut hook_map = crate::bds::hook::HOOK_MAP.lock().expect("Failed to get HOOK_MAP");
        hook_map.insert(String::from(#clean_name), hook);
    };

    proc_macro::TokenStream::from(tokens)
}