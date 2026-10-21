//! Noti DSL module containing AST types.
//!
//! The result of parsing is an AST representing all declarations in the input.
//! The AST represents the parsed input, but does not guarantee its semantic validity.

/// Represents an arbitrary scope containing declarations.
///
/// A "scope" in the Noti DSL is not only a block enclosed in curly braces, but also an abstract
/// syntax node that introduces a new scope (e.g., a widget can introduce a new scope).
///
/// Declarations within a scope can be arbitrary, but some of them might be semantically invalid.
#[derive(PartialEq, Eq)]
pub struct ScopeDeclaration {
    pub kind: ScopeKind,
    pub children: Vec<InScopeDeclaration>,
}

/// Represents a declaration that appears within a scope.
///
/// Currently, declarations are accessible only within a scope and in sequential order. A declaration
/// can access previous declarations, but not subsequent ones.
#[derive(PartialEq, Eq)]
pub enum InScopeDeclaration {
    Alias(AliasDeclaration),
    State(StateDeclaration),
    Widget(ScopeDeclaration),
}

/// Represents the kind of a scope.
///
/// Scopes differ depending on who introduces them and how.
#[derive(PartialEq, Eq)]
pub enum ScopeKind {
    /// The global scope, essentially the top-level scope of the input.
    Global,

    /// The scope of a widget.
    Widget(WidgetDeclaration),
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
#[derive(PartialEq, Eq)]
pub struct AliasDeclaration {
    pub name: String,
    pub replacement: AliasReplacement,
}

/// Represents the right-hand value of an alias.
#[derive(PartialEq, Eq)]
pub enum AliasReplacement {
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
    Path(TypePath),
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
#[derive(PartialEq, Eq)]
pub struct WidgetDeclaration {
    pub type_path: TypePath,
    pub fields: Vec<Field>,
}

/// Represents a path to a type, including widgets.
///
/// Uses Rust-like syntax (e.g., `path::to::SomeWidget`) and contains only a vector of path elements
/// (using the same example: `["path", "to", "SomeWidget"]`).
#[derive(Clone, PartialEq, Eq)]
pub struct TypePath(pub Vec<String>);

/// Represents a field in a type, including widgets.
///
/// Example of a field declaration:
/// ```noti
/// field_name = "String"
/// // Or
/// size = 4
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub value: FieldValue,
}

/// Represents the right-hand value assigned to a field.
///
/// A right-hand value can belong to one of three categories: literals, types, or states.
#[derive(Clone, PartialEq, Eq)]
pub enum FieldValue {
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
#[derive(PartialEq, Eq)]
pub struct StateDeclaration {
    pub name: String,
    pub value: Literal,
}

/// Represents a literal value that can be assigned to something.
#[derive(Clone, PartialEq, Eq)]
pub enum Literal {
    /// A string literal from the input, preserved as written, including double quotes.
    QuotedString(String),

    /// A `u64` literal from the input.
    UInt(u64),
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
#[derive(Clone, PartialEq, Eq)]
pub struct TypeDeclaration {
    pub type_path: TypePath,
    pub fields: Vec<Field>,
}
