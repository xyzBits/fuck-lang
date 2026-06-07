

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn time_it(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_name = &input_fn.sig.ident;
    let fn_block = &input_fn.block;

    let expended = quote! {
      #fn_vis #fn_sig {
            let start_time = std::time::Instant::now();

            let result = (|| {
                #fn_block
            })();

            let duration = start_time.elapsed();

            println!("Time it fn:{} took {}ms",stringify!(#fn_name), duration.as_millis());
        }
    };

    expended.into()
}