// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
	parse_macro_input,
	parse_quote,
	spanned::Spanned,
	Attribute,
	Data,
	DataEnum,
	DataStruct,
	DeriveInput,
	Error,
	Fields,
	GenericParam,
	Index,
	Meta,
	Type,
	Variant,
};

use crate::numbers_to_words::encode_ordinal;

mod numbers_to_words;

/// Derives [`Default`] with more flexibility than Rust's built-in `#[derive(Default)]`.
///
/// In addition to the functionality allowed by the built-in `#[derive(Default)]`, this version
/// allows:
/// - overriding the default values for particular fields using `#[default = ...]` or
///   `#[default(...)]`; and
/// - using `#[default]` on a non-unit enum variant
///
/// # Examples
///
/// ## Structs
/// ```
/// # #[allow(unused_qualifications)]
/// #[derive(derive_extras::Default)]
/// struct ExampleStruct {
///     x: i32,
///     #[default = 15]
///     y: i32,
///     z: i32,
/// }
/// ```
/// For this struct, the following [`Default`] implementation is derived:
/// ```
/// # struct ExampleStruct {
/// #     x: i32,
/// #     y: i32,
/// #     z: i32,
/// # }
/// #
/// impl Default for ExampleStruct {
///     fn default() -> Self {
///         Self {
///             x: Default::default(),
///             y: 15,
///             z: Default::default(),
///         }
///     }
/// }
/// ```
///
/// ## Enums
/// ```
/// # #[allow(unused_qualifications)]
/// #[derive(derive_extras::Default)]
/// enum ExampleEnum {
///     Unit,
///     Tuple(i32, i32, i32),
///
///     #[default]
///     Struct {
///         x: i32,
///         #[default = 15]
///         y: i32,
///         z: i32,
///     },
/// }
/// ```
/// For this enum, the following [`Default`] implementation is derived:
/// ```
/// # enum ExampleEnum {
/// #     Unit,
/// #     Tuple(i32, i32, i32),
/// #     Struct {
/// #         x: i32,
/// #         y: i32,
/// #         z: i32,
/// #     },
/// # }
/// #
/// impl Default for ExampleEnum {
///     fn default() -> Self {
///         Self::Struct {
///             x: Default::default(),
///             y: 15,
///             z: Default::default(),
///         }
///     }
/// }
/// ```
///
/// [`Default`]: core::default::Default
#[proc_macro_derive(Default, attributes(default))]
pub fn default(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let DeriveInput {
		mut generics,
		ident,
		data,
		..
	} = input;

	for param in &mut generics.params {
		if let GenericParam::Type(param) = param {
			param.bounds.push(parse_quote!(::core::default::Default))
		}
	}

	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let body = match data {
		Data::Struct(DataStruct { fields, .. }) => {
			let init = expand_default_for_fields(fields);

			quote!(Self #init)
		},

		Data::Enum(r#enum) => expand_default_for_enum(r#enum).unwrap_or_else(|error| error.into_compile_error()),

		Data::Union(_) => unimplemented!("unions are not supported"),
	};

	let tokens = quote! {
		impl #impl_generics ::core::default::Default for #ident #ty_generics #where_clause {
			fn default() -> Self {
				#body
			}
		}
	};

	tokens.into()
}

fn expand_default_for_enum(r#enum: DataEnum) -> syn::Result<TokenStream2> {
	let variant = {
		let mut variant = None;

		for var in r#enum.variants {
			for attr in &var.attrs {
				if attr.meta.require_path_only().map(|path| path.is_ident("default"))? {
					match &variant {
						None => variant = Some(var),
						Some(_) => return Err(Error::new(attr.span(), "conflicting #[default] attribute")),
					}

					break;
				}
			}
		}

		variant
	};

	match variant {
		Some(Variant { ident, fields, .. }) => {
			let init = expand_default_for_fields(fields);

			Ok(quote!(Self::#ident #init))
		},

		None => Err(Error::new(
			Span::call_site(),
			"one enum variant must have a #[default] attribute",
		)),
	}
}

fn expand_default_for_fields(fields: Fields) -> Option<TokenStream2> {
	match fields {
		Fields::Unit => None,

		Fields::Named(fields) => {
			let field_init = fields.named.into_pairs().map(|pair| {
				let (field, comma) = pair.into_tuple();

				let (ident, ty, attrs) = (field.ident, field.ty, field.attrs);

				let value = field_value(ty, attrs);

				quote!(#ident: #value #comma)
			});

			Some(quote!({ #(#field_init)* }))
		},

		Fields::Unnamed(fields) => {
			let field_init = fields.unnamed.into_pairs().map(|pair| {
				let (field, comma) = pair.into_tuple();

				let value = field_value(field.ty, field.attrs);

				quote!(#value #comma)
			});

			Some(quote!((#(#field_init)*)))
		},
	}
}

fn field_value(ty: Type, attrs: Vec<Attribute>) -> TokenStream2 {
	let default_attr = attrs
		.into_iter()
		.find(|attribute| attribute.meta.path().is_ident("default"));

	match default_attr {
		// If there is a default attribute, use its value.
		Some(default_attr) => match default_attr.meta {
			Meta::Path(path) => Error::new(
				path.span(),
				format!(
					"expected a value for this attribute: `{}(...)` or `{} = ...`",
					"default", "default",
				),
			)
			.into_compile_error(),

			Meta::List(meta) => {
				let tokens = meta.tokens;

				quote!({ #tokens })
			},

			Meta::NameValue(meta) => meta.value.into_token_stream(),
		},

		// If there is no default attribute, use `Default::default()`.
		None => {
			quote!(<#ty as ::core::default::Default>::default())
		},
	}
}

/// Generates appropriate builder methods for a struct.
///
/// This assumes that the struct itself acts like a builder.
///
/// This derive macro also adds a `#[new]` helper attribute. If this is added to the struct, a `new`
/// function is also generated with a `where Self: Default` bound.
///
/// The builder methods generated for each field will have that field's visibility: private fields
/// will have private methods, etc.
///
/// # Examples
/// ```
/// # use derive_extras::builder;
/// #
/// #[derive(Default, builder)]
/// #[new]
/// struct Example {
///     pub x: i32,
///     pub y: i32,
/// }
/// ```
/// This will derive the following implementations:
/// ```
/// # #[derive(Default)]
/// # struct Example {
/// #     pub x: i32,
/// #     pub y: i32,
/// # }
/// #
/// impl Example {
///     /// Creates a new `Example`.
///     ///
///     /// This is equivalent to <code>Example::[default()]</code>.
///     ///
///     /// [default()]: Default::default()
///     pub fn new() -> Self
///     where
///         Self: Default,
///     {
///         Self::default()
///     }
///
///     /// Sets `x` to the given value.
///     pub fn x(mut self, x: i32) -> Self {
///         self.x = x;
///
///         self
///     }
///
///     /// Sets `y` to the given value.
///     pub fn y(mut self, y: i32) -> Self {
///         self.y = y;
///
///         self
///     }
/// }
/// ```
///
///
/// `#[derive(builder)]` also works on tuple structs (with any number of fields):
/// ```
/// # use derive_extras::builder;
/// #
/// #[derive(Default, builder)]
/// struct Example(pub i32, pub i32);
/// ```
/// This will derive the following implementations:
/// ```
/// # #[derive(Default)]
/// # struct Example(pub i32, pub i32);
/// #
/// impl Example {
///     /// Sets the first field to the given value.
///     pub fn first(mut self, first: i32) -> Self {
///         self.0 = first;
///
///         self
///     }
///
///     /// Sets the second field to the given value.
///     pub fn second(mut self, second: i32) -> Self {
///         self.1 = second;
///
///         self
///     }
/// }
/// ```
#[proc_macro_derive(builder, attributes(new))]
pub fn builder(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let tokens = match input.data {
		Data::Struct(r#struct) => {
			let name = input.ident;

			let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

			let new = input.attrs.iter().any(|attr| attr.path().is_ident("new")).then(|| {
				quote! {
					#[doc = concat!("Creates a new `", stringify!(#name), "`.")]
					#[doc = ""]
					#[doc = concat!("This is equivalent to <code>", stringify!(#name), "::[default()]</code>.")]
					#[doc = ""]
					#[doc = "[default()]: ::core::default::Default::default()"]
					pub fn new() -> Self
					where
						Self: ::core::default::Default,
					{
						<Self as ::core::default::Default>::default()
					}
				}
			});

			let config_methods = match &r#struct.fields {
				Fields::Unit => None,

				// Named fields.
				Fields::Named(fields) => {
					let methods: TokenStream2 = fields
						.named
						.iter()
						.map(|field| {
							let vis = &field.vis;
							let ty = &field.ty;

							let ident = &field.ident;

							let docs = ident
								.as_ref()
								.map(|ident| format!("Sets `{ident}` to the given value."));

							quote! {
								#[doc = #docs]
								#vis fn #ident(mut self, #ident: #ty) -> Self {
									self.#ident = #ident;

									self
								}
							}
						})
						.collect();

					Some(methods)
				},

				// Unnamed fields.
				Fields::Unnamed(fields) => {
					let methods = fields
						.unnamed
						.iter()
						.enumerate()
						.map(|(i, field)| {
							let vis = &field.vis;
							let ty = &field.ty;

							let index = Index::from(i);
							let ident = Ident::new(&encode_ordinal(i + 1, '_'), Span::call_site());

							let ordinal = encode_ordinal(i + 1, ' ');
							let docs = format!("Sets the {ordinal} field to the given value.");

							quote! {
								#[doc = #docs]
								#vis fn #ident(mut self, #ident: #ty) -> Self {
									self.#index = #ident;

									self
								}
							}
						})
						.collect();

					Some(methods)
				},
			};

			// Final generated implementation.
			quote! {
				impl #impl_generics #name #ty_generics #where_clause {
					#new

					#config_methods
				}
			}
		},

		Data::Enum(_enum) => unimplemented!("enums are not supported"),
		Data::Union(_union) => unimplemented!("unions are not supported"),
	};

	tokens.into()
}
