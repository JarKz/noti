//! Noti DSL module containing AST types.
//!
//! The result of parsing is an AST representing all declarations in the input.
//! The AST represents the parsed input, but does not guarantee its semantic validity.

use widgets::types::Bgra;

/// The spanned node, containing information about the node and its position in an input.
///
/// It's not meaningful outside of DSL, and carries information only for pointing to an exact
/// location in an input.
#[derive(Clone, PartialEq, Debug)]
pub struct Spanned<T> {
    span: Span,
    value: T,
}

impl<T> Spanned<T> {
    pub fn new(value: T, start: usize, end: usize) -> Self {
        Self {
            value,
            span: Span { start, end },
        }
    }

    pub fn clone_value(&self) -> T
    where
        T: Clone,
    {
        self.value.clone()
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn relocate(self, span: Span) -> Self {
        Self {
            value: self.value,
            span,
        }
    }

    pub fn unspan(self) -> T {
        self.value
    }

    pub fn map<F, R>(self, f: F) -> Spanned<R>
    where
        F: FnOnce(T) -> R,
    {
        Spanned {
            value: f(self.value),
            span: self.span
        }
    }
}

impl<T> Spanned<Option<T>> {
    pub fn rewrap_option(self) -> Option<Spanned<T>> {
        match self.value {
            Some(value) => Some(Spanned {
                span: self.span,
                value,
            }),
            None => None,
        }
    }
}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::ops::DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

/// The spatial information containing the start and the end (exclusive) positions.
///
/// Usually the start is the offset from begin of an input and the end is the start + a length of
/// word.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Represents an arbitrary scope containing declarations.
///
/// A "scope" in the Noti DSL is not only a block enclosed in curly braces, but also an abstract
/// syntax node that introduces a new scope (e.g., a widget can introduce a new scope).
///
/// Declarations within a scope can be arbitrary, but some of them might be semantically invalid.
#[derive(PartialEq, Debug)]
pub(crate) struct ScopeDeclaration {
    pub kind: ScopeKind,
    pub children: Vec<Spanned<InScopeDeclaration>>,
}

/// Represents a declaration that appears within a scope.
///
/// Currently, declarations are accessible only within a scope and in sequential order. A declaration
/// can access previous declarations, but not subsequent ones.
#[derive(PartialEq, Debug)]
pub(crate) enum InScopeDeclaration {
    Alias(AliasDeclaration),
    State(StateDeclaration),
    Widget(ScopeDeclaration),
}

/// Represents the kind of a scope.
///
/// Scopes differ depending on who introduces them and how.
#[derive(PartialEq, Debug)]
pub(crate) enum ScopeKind {
    /// The global scope, essentially the top-level scope of the input.
    Global,

    /// The scope of a widget.
    Widget(Spanned<WidgetDeclaration>),
}

/// Represents an alias declared in the input.
///
/// An alias has a name rather than a type path. When the alias is used, the name is replaced with
/// the declared right-hand value (an alias replacement).
///
/// Example of an alias declaration:
/// ```noti
/// alias Row = FlexBox(direction = "horizontal") {};
/// ```
#[derive(PartialEq, Debug)]
pub(crate) struct AliasDeclaration {
    pub name: Spanned<String>,
    pub replacement: Spanned<AliasReplacement>,
}

/// Represents the right-hand value of an alias.
#[derive(PartialEq, Debug)]
pub(crate) enum AliasReplacement {
    /// A right-hand value declared as a widget, with curly braces at the end.
    ///
    /// In the resulting widget, fields can be overridden, but children cannot be overridden directly.
    /// If the widget has children, they are copied to the actual location, where additional children
    /// can be appended.
    Widget(ScopeDeclaration),

    /// A right-hand value declared as a type.
    ///
    /// The resulting type can be used to override fields.
    Type(TypeDeclaration),

    /// A right-hand value declared as a path.
    ///
    /// The resulting path can be used to build a type or a widget.
    Path(TypePath<Spanned<String>>),
}

/// Represents a widget declaration.
///
/// > **NOTE**: [`WidgetDeclaration`] and [`TypeDeclaration`] are similar, but they are not the same
/// > thing. A widget declaration represents a widget that introduces a new scope, whereas a type
/// > declaration does not introduce a new scope.
///
/// A widget can be accessed by its type path.
///
/// Example of a widget declaration:
/// ```noti
/// FlexBox(
///     direction = "horizontal"
/// ) {
///     /* children of the FlexBox go here */
/// }
/// ```
#[derive(PartialEq, Debug)]
pub(crate) struct WidgetDeclaration {
    pub type_path: Spanned<TypePath<Spanned<String>>>,
    pub fields: Vec<Spanned<Field>>,
}

/// Represents a path to a type, including widgets.
///
/// Uses Rust-like syntax (e.g., `path::to::SomeWidget`) and contains only a vector of path elements
/// (using the same example: `["path", "to", "SomeWidget"]`).
#[derive(Clone, PartialEq, Debug)]
pub struct TypePath<T>(pub Vec<T>);

impl TypePath<Spanned<String>> {
    pub fn unspan_cloned(&self) -> TypePath<String> {
        TypePath(self.0.iter().map(|val| (**val).clone()).collect())
    }
}

impl std::fmt::Display for TypePath<String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("::"))
    }
}

impl std::fmt::Display for TypePath<Spanned<String>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|val| &**val)
                .fold(String::new(), |mut acc, ident| {
                    if !acc.is_empty() {
                        acc.push_str("::");
                    }

                    acc.push_str(ident);
                    acc
                })
        )
    }
}

/// Represents a field in a type, including widgets.
///
/// Example of a field declaration:
/// ```noti
/// field_name = "String"
/// // Or
/// size = 4
/// ```
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Field {
    pub name: Spanned<String>,
    pub value: Spanned<FieldValue>,
}

/// Represents the right-hand value assigned to a field.
///
/// A right-hand value can belong to one of three categories: literals, types, or states.
#[derive(Clone, PartialEq, Debug)]
pub(crate) enum FieldValue {
    State(String),
    Literal(Literal),
    Type(TypeDeclaration),
}

/// Represents a state declared in the input.
///
/// States are like variables in UI code, so they are treated like regular variables.
///
/// Example of a state declaration:
/// ```noti
/// state some_state = 3;
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct StateDeclaration {
    pub name: Spanned<String>,
    pub value: Spanned<Literal>,
}

/// Represents a literal value that can be assigned to something.
#[derive(Clone, PartialEq, Debug)]
pub enum Literal {
    /// A string literal from the input.
    String(String),

    /// A `u64` literal from the input.
    UInt(u64),

    /// A `bool` literal from the input.
    Bool(bool),

    /// A color literal, usually represents a fill color or gradient.
    Color(Color),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Color {
    Rgba(Bgra<u8>),
    LinearGradient { angle: i16, colors: Vec<Bgra<u8>> },
}

/// Represents a type declaration.
///
/// > **NOTE**: [`WidgetDeclaration`] and [`TypeDeclaration`] are similar, but they are not the same
/// > thing. A widget declaration represents a widget that introduces a new scope, whereas a type
/// > declaration does not introduce a new scope.
///
/// A type can be accessed by its type path.
///
/// Example of a type declaration:
/// ```noti
/// Border(size = 8)
/// ```
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct TypeDeclaration {
    pub type_path: Spanned<TypePath<Spanned<String>>>,
    pub fields: Vec<Spanned<Field>>,
}
