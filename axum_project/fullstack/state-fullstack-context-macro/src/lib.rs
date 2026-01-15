use proc_macro::TokenStream;
use quote::quote;
use syn::{DataStruct, DeriveInput, Fields, FieldsNamed, parse_macro_input};

#[proc_macro_derive(FromFullstackContextRef)]
pub fn __from_full_stack_context_ref(item: TokenStream) -> TokenStream {
    // 이해하기 쉬운 Derive용 토큰을 추출 (구조체, 열거형, 공용체) 이하는 구조체를 가정
    let ast = parse_macro_input!(item as DeriveInput);
    // 구조체 이름 추출
    let st_name = ast.ident;

    // 구조체에서 이름있는 구조체 필드를 추출
    let fields = match ast.data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => unimplemented!("only works for struct with named fields"),
    };

    // 기존 이름의 FromRef구현
    let base_macro = quote! {
        impl axum::extract::FromRef<dioxus::fullstack::FullstackContext> for #st_name {
            fn from_ref(state: &dioxus::fullstack::FullstackContext) -> Self {
                state.extension::<#st_name>().unwrap().clone()
            }
        }
    };

    // 중복타입을 제외하더라도, 중복이 존재하면 첫번째 필드만 가져오기 때문에 문제가 생김, 차라리 에러를 발생시켜서, 중복 타입 필드가 존재하지 않게끔 해주는게 좋을 것 같음
    // let mut set = HashSet::new();

    // 필드의 FromRef구현
    let builder_fields = fields
        .iter()
        // .filter(|f| {
        //     let ty = &f.ty;
        //     let ty_str = quote! {#ty}.to_string();
        //     set.insert(ty_str)
        // })
        .map(|f| {
            let name = &f.ident;
            let ty = &f.ty;
            quote! {
                impl axum::extract::FromRef<dioxus::fullstack::FullstackContext> for #ty {
                    fn from_ref(state: &dioxus::fullstack::FullstackContext) -> Self {
                        let p = state.extension::<#st_name>().expect("This Struct field must differnt type about other filed!, and most be a new type pattern");
                        p.#name.clone()
                    }
                }
            }
        });

    // 반환
    let res = quote! {
        #base_macro
        #(#builder_fields)*
    };

    res.into()
}
