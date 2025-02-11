use proc_macro::TokenStream;
use proc_macro_error2::abort_call_site;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Mmio)]
pub fn derive_answer_fn(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let wrapper_ident = format_ident!("Mmio{}", ident);
    let Data::Struct(ref s) = input.data else {
        abort_call_site!("`#[derive(Mmio)]` only supports struct");
    };

    let fields = &s.fields;

    let Fields::Named(ref fields) = fields else {
        abort_call_site!("`#[derive(Mmio)]` only supports structs with named fields");
    };

    let field_funcs = fields.named.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        // TODO: check the ident here. If it's _xxx, then don't emit functions
        let read_fn_name = format_ident!("read_{}", ident);
        let write_fn_name = format_ident!("write_{}", ident);
        let modify_fn_name = format_ident!("modify_{}", ident);
        let ty = &field.ty;
        // TODO: check the type here. If it's an array, we need an array function
        quote! {
            fn #read_fn_name(&mut self) -> #ty {
                let addr = unsafe { &raw mut ((*self.ptr).#ident) };
                unsafe {
                    addr.read_volatile()
                }
            }

            fn #write_fn_name(&mut self, value: #ty) {
                let addr = unsafe { &raw mut ((*self.ptr).#ident) };
                unsafe {
                    addr.write_volatile(value)
                }
            }

            fn #modify_fn_name<F>(&mut self, f: F) where F: FnOnce(#ty) -> #ty {
                let value = self. #read_fn_name();
                let new_value = f(value);
                self. #write_fn_name(new_value);
            }
        }
    });

    let expanded = quote! {
        struct #wrapper_ident {
            ptr: *mut #ident
        }

        impl #wrapper_ident {
            #(#field_funcs)*
        }

        impl #ident {
            fn mmio(ptr: *mut #ident) -> #wrapper_ident {
                #wrapper_ident {
                    ptr
                }
            }
        }
    };

    TokenStream::from(expanded)
}
