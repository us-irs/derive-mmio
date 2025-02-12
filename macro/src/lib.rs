//! The derive macro for the Mmio crate.

use proc_macro::TokenStream;
use proc_macro_error2::{abort_call_site, proc_macro_error};
use quote::{format_ident, quote};
use syn::{parse_macro_input, punctuated::Punctuated, Data, DeriveInput, Fields, Meta, Token};

#[proc_macro_error]
#[proc_macro_derive(Mmio)]
pub fn derive_mmio(input: TokenStream) -> TokenStream {
    // validate our input
    let input = parse_macro_input!(input as DeriveInput);
    let mut is_repr_c = false;
    'attr: for attr in input.attrs.iter() {
        if attr.path().is_ident("repr") {
            let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).unwrap();
            for meta in nested {
                if let Meta::Path(path) = meta {
                    if path.is_ident("C") {
                        is_repr_c = true;
                        break 'attr;
                    }
                }
            }
        }
    }
    if !is_repr_c {
        abort_call_site!("`#[derive(Mmio)]` only works on repr(C) types");
    }
    let ident = input.ident;
    let wrapper_ident = format_ident!("Mmio{}", ident);
    let Data::Struct(ref s) = input.data else {
        abort_call_site!("`#[derive(Mmio)]` only supports struct");
    };
    let Fields::Named(ref fields) = &s.fields else {
        abort_call_site!("`#[derive(Mmio)]` only supports structs with named fields");
    };

    // process the input to create the fragments we want
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

    // combine the fragments into the desired output code
    TokenStream::from(quote! {
        struct #wrapper_ident {
            ptr: *mut #ident
        }

        impl #wrapper_ident {
            #(#field_funcs)*
        }

        impl #ident {
            #[doc = "Create a new handle to this peripheral."]
            #[doc = ""]
            #[doc = "# Safety"]
            #[doc = ""]
            #[doc = "The pointer given must have suitable alignment, and point to an object"]
            #[doc = "which matches the layout given by the structure pointed to."]
            #[doc = ""]
            #[doc = "If you create multiple instances of this handle at the same time,"]
            #[doc = "you are responsible for ensuring that there are no read-modify-write"]
            #[doc = "races on any of the registers."]
            pub unsafe fn new_mmio(ptr: *mut #ident) -> #wrapper_ident {
                #wrapper_ident {
                    ptr
                }
            }

            #[doc = "Create a new handle to this peripheral given an address."]
            #[doc = ""]
            #[doc = "# Safety"]
            #[doc = ""]
            #[doc = "See the safety notes for [`new_mmio`]. In addition, the address given"]
            #[doc = "must have [exposed provenance](https://doc.rust-lang.org/stable/std/ptr/fn.with_exposed_provenance_mut.html)."]
            pub unsafe fn new_mmio_at(addr: usize) -> #wrapper_ident {
                #wrapper_ident {
                    // TODO: require Rust 1.84. Check Rust version and offer
                    // plain pointer cast here.
                    ptr: core::ptr::with_exposed_provenance_mut(addr)
                }
            }
        }
    })
}
