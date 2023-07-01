use darling::ToTokens;
use syn::{ImplGenerics, TypeGenerics, WhereClause};

#[derive(Debug)]
pub(crate) struct OwnedGenerics {
    generics: syn::Generics,
    imp_copy: syn::Generics,
}

impl OwnedGenerics {
    pub(crate) fn new(generics: syn::Generics) -> Self {
        Self {
            generics: generics.clone(),
            imp_copy: generics,
        }
    }

    pub(crate) fn split_for_impl(
        &self,
    ) -> (ImplGenerics, TypeGenerics, Option<&WhereClause>) {
        let (_, ty, wher) = self.generics.split_for_impl();
        let (imp, _, _) = self.imp_copy.split_for_impl();

        (imp, ty, wher)
    }

    pub(crate) fn push_impl_param(
        &mut self,
        param: syn::TypeParam,
    ) -> syn::Type {
        let ident = param.ident.clone();

        self.imp_copy.params.push(syn::GenericParam::Type(param));

        syn::Type::Verbatim(ident.to_token_stream())
    }
}
