use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;

pub(crate) fn resolve_dsir_path() -> syn::Result<syn::Path> {
    match crate_name("dsir") {
        // `crate` fails in examples/binaries inside the dsir package because
        // there it points at the example crate, not the library. Use the crate
        // alias (`extern crate self as dsir`) for a stable path.
        Ok(FoundCrate::Itself) => Ok(syn::parse_quote!(::dsir)),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name.replace('-', "_"), Span::call_site());
            Ok(syn::parse_quote!(::#ident))
        }
        Err(_) => Err(syn::Error::new(
            Span::call_site(),
            "could not resolve `dsir`; add it as a dependency (renamed dependencies are supported)",
        )),
    }
}
