use std::collections::HashMap;

use crate::{
    analysis::{CompiledFieldValue, CompiledTypeDeclaration, DiagnosticLabel},
    ast::{self, Span, Spanned},
    plugin::{
        types::{
            AlignmentBuilder, AnimationDefinitionBuilder, BorderBuilder, FontBuilder,
            SpacingBuilder, SpatialChangeDefinitionBuilder,
        },
        widgets::{
            AnimatedVisibilityBuilder, BoxBuilder, ConstrainedBoxBuilder, FlexBoxBuilder,
            ImageBuilder, TextBuilder,
        },
    },
};

pub mod types;
pub mod widgets;

#[derive(Default)]
pub struct BuildContext {
    plugins: HashMap<String, Box<dyn TreeBuilder>>,
}

impl BuildContext {
    pub fn with_default_plugins(mut self) -> Self {
        let default_plugins: Vec<(String, Box<dyn TreeBuilder>)> = vec![
            // ----------------- Types
            ("Border".to_owned(), Box::new(BorderBuilder)),
            ("Alignment".to_owned(), Box::new(AlignmentBuilder)),
            ("Spacing".to_owned(), Box::new(SpacingBuilder)),
            ("Font".to_owned(), Box::new(FontBuilder)),
            (
                "AnimationDefinition".to_owned(),
                Box::new(AnimationDefinitionBuilder),
            ),
            (
                "SpatialChangeDefinition".to_owned(),
                Box::new(SpatialChangeDefinitionBuilder),
            ),
            // ---------------- Widgets
            (
                "AnimatedVisibility".to_owned(),
                Box::new(AnimatedVisibilityBuilder),
            ),
            ("FlexBox".to_owned(), Box::new(FlexBoxBuilder)),
            ("Box".to_owned(), Box::new(BoxBuilder)),
            ("ConstrainedBox".to_owned(), Box::new(ConstrainedBoxBuilder)),
            ("Text".to_owned(), Box::new(TextBuilder)),
            ("Image".to_owned(), Box::new(ImageBuilder)),
        ];

        self.plugins.extend(default_plugins);
        self
    }

    pub fn add_plugin(&mut self, plugin: Plugin) -> Result<(), anyhow::Error> {
        if let std::collections::hash_map::Entry::Vacant(e) = self.plugins.entry(plugin.type_path) {
            e.insert(plugin.builder);
            Ok(())
        } else {
            Err(anyhow::Error::msg(
                "Plugin type path clashes with existing one.",
            ))
        }
    }

    pub(crate) fn builder_by_type_path(&self, type_path: &str) -> Option<&dyn TreeBuilder> {
        self.plugins.get(type_path).map(|builder| &**builder)
    }
}

pub struct Plugin {
    type_path: String,
    builder: Box<dyn TreeBuilder>,
}

impl Plugin {
    /// Creates a plugin for DSL system.
    ///
    /// The type_path must be a valid path containting identifiers, delimited by "::" (i.e.
    /// "custom::Widget").
    pub fn new(type_path: String, builder: impl TreeBuilder + 'static) -> Self {
        Self {
            type_path,
            builder: Box::new(builder),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ValidationError {
    kind: ValidationErrorKind,
    message: String,
    labels: Vec<DiagnosticLabel>,
}

#[derive(Debug, PartialEq)]
pub enum ValidationErrorKind {
    MissingFields,
    UnknownField,
    InvalidFieldValue,
    MismatchedFieldValueType,
    MisusedFieldName,
    IncompatibleStateType,
    MissingBuilderImplementation,
}

impl ValidationError {
    pub(crate) fn make_missing_builder_implementation(
        compiled_type_declaration: &CompiledTypeDeclaration,
    ) -> Self {
        ValidationError {
            kind: ValidationErrorKind::MissingBuilderImplementation,
            message: format!(
                "{} type is unrecognized, possibly a tree builder plugin is absent",
                compiled_type_declaration.type_path.clone_value()
            ),
            labels: vec![DiagnosticLabel {
                span: *compiled_type_declaration.type_path.span(),
                message: format!(
                    "{} type is unknown",
                    compiled_type_declaration.type_path.clone_value()
                ),
            }],
        }
    }

    fn make_mismatched_field_type(
        field_name: &str,
        field_value_span: Span,
        expected_type: &str,
    ) -> Self {
        Self {
            kind: ValidationErrorKind::MismatchedFieldValueType,
            message: format!("Mismatched type for {field_name}"),
            labels: vec![DiagnosticLabel {
                message: format!("Mismatched type for {field_name}: expected {expected_type}"),
                span: field_value_span,
            }],
        }
    }
}

pub trait TreeBuilder {
    fn required_fields(&self) -> &'static [&'static str];
    fn available_fields(&self) -> &'static [&'static str];
    fn kind(&self) -> TreeNodeKind;

    fn validate(
        &self,
        type_declaration: &CompiledTypeDeclaration,
        build_context: &BuildContext,
    ) -> Result<(), ValidationError> {
        let missing_fields: Vec<&str> = self
            .required_fields()
            .iter()
            .filter(|&&field_name| !type_declaration.fields.contains_key(field_name))
            .map(ToOwned::to_owned)
            .collect();

        if !missing_fields.is_empty() {
            return Err(ValidationError {
                kind: ValidationErrorKind::MissingFields,
                message: format!(
                    "Missing fields for type {}",
                    type_declaration.type_path.clone_value()
                ),
                labels: vec![DiagnosticLabel {
                    span: *type_declaration.type_path.span(),
                    message: format!("Use {} fields for this type", missing_fields.join(", ")),
                }],
            });
        }

        let kind = self.kind();
        let mut available_fields = self.available_fields().to_vec();
        available_fields.extend(kind.available_fields());

        for (field_name, field_value) in &type_declaration.fields {
            if !available_fields.contains(&&**field_name) {
                return Err(ValidationError {
                    kind: ValidationErrorKind::UnknownField,
                    message: format!(
                        "One or more fields is unknown for type {}",
                        type_declaration.type_path.clone_value()
                    ),
                    labels: vec![DiagnosticLabel {
                        span: *type_declaration.get_field_span(field_name).expect(
                            "The field span usually persist on creation of CompiledTypeDeclaration",
                        ),
                        message: format!("The {field_name} is unknown for this type"),
                    }],
                });
            }

            match &**field_value {
                CompiledFieldValue::State(_) | CompiledFieldValue::Literal(_) => (),
                CompiledFieldValue::Type(compiled_type_declaration) => {
                    let Some(builder) = build_context
                        .builder_by_type_path(&compiled_type_declaration.type_path.to_string())
                    else {
                        return Err(ValidationError::make_missing_builder_implementation(
                            compiled_type_declaration,
                        ));
                    };

                    builder.validate(compiled_type_declaration, build_context)?;
                }
            }
        }

        kind.validate_type(type_declaration)?;
        self.on_validate(type_declaration)?;

        Ok(())
    }

    fn on_validate(
        &self,
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError>;

    fn build(&self) /* -> Option<Widget> */;
}

pub enum TreeNodeKind {
    Type,
    Widget(WidgetKind),
}

impl TreeNodeKind {
    fn available_fields(&self) -> &'static [&'static str] {
        match self {
            TreeNodeKind::Type => &[],
            TreeNodeKind::Widget(widget_kind) => widget_kind.available_fields(),
        }
    }

    fn validate_type(
        &self,
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        match self {
            TreeNodeKind::Type => Ok(()),
            TreeNodeKind::Widget(widget_kind) => widget_kind.validate_widget(type_declaration),
        }
    }
}

pub enum WidgetKind {
    Minimal,
    Standard,
    Container,
}

impl WidgetKind {
    fn available_fields(&self) -> &'static [&'static str] {
        match self {
            WidgetKind::Minimal => &["key", "input_behavior"],
            WidgetKind::Standard => &["key", "input_behavior", "margin"],
            WidgetKind::Container => &[
                "key",
                "input_behavior",
                "margin",
                "padding",
                "background_color",
                "border",
                "alignment",
            ],
        }
    }

    fn validate_widget(
        &self,
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        match self {
            WidgetKind::Minimal => Self::validate_minimal_fields(type_declaration),
            WidgetKind::Standard => {
                Self::validate_minimal_fields(type_declaration)?;
                Self::validate_standard_fields(type_declaration)
            }
            WidgetKind::Container => {
                Self::validate_minimal_fields(type_declaration)?;
                Self::validate_standard_fields(type_declaration)?;
                Self::validate_container_fields(type_declaration)
            }
        }
    }

    fn validate_minimal_fields(
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        if let Some(key_value) = type_declaration.fields.get("key") {
            expect_string_literal("key", key_value, StringVerifier::AcceptAny)?;
        }

        if let Some(input_behavior_value) = type_declaration.fields.get("input_behavior") {
            expect_string_literal(
                "input_behavior",
                input_behavior_value,
                &["accepts", "passes-to-children"][..],
            )?;
        }

        Ok(())
    }

    fn validate_standard_fields(
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        if let Some(margin_value) = type_declaration.fields.get("margin") {
            expect_type(
                "margin",
                margin_value,
                ast::TypePath(vec!["Spacing".to_owned()]),
            )?;
        }

        Ok(())
    }

    fn validate_container_fields(
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        if let Some(padding_value) = type_declaration.fields.get("padding") {
            expect_type(
                "padding",
                padding_value,
                ast::TypePath(vec!["Spacing".to_owned()]),
            )?;
        }

        if let Some(border_value) = type_declaration.fields.get("border") {
            expect_type(
                "border",
                border_value,
                ast::TypePath(vec!["Border".to_owned()]),
            )?;
        }

        if let Some(background_color_value) = type_declaration.fields.get("background_color") {
            expect_color_literal("background_color", background_color_value)?;
        }

        if let Some(alignment_value) = type_declaration.fields.get("alignment") {
            expect_type(
                "alignment",
                alignment_value,
                ast::TypePath(vec!["Alignment".to_owned()]),
            )?;
        }

        Ok(())
    }
}

pub fn expect_string_literal<'a: 'static, V>(
    field_name: &str,
    field_value: &Spanned<CompiledFieldValue>,
    verifier: V,
) -> Result<(), ValidationError>
where
    V: Into<StringVerifier<'a>>,
{
    match &**field_value {
        CompiledFieldValue::Literal(literal) => match literal {
            ast::Literal::String(value) => {
                match verifier.into() {
                    StringVerifier::PossibleValues(values) => {
                        if !values.contains(&&**value) {
                            return Err(ValidationError {
                                kind: ValidationErrorKind::InvalidFieldValue,
                                message: format!("Invalid value for {field_name}"),
                                labels: vec![DiagnosticLabel {
                                    span: *field_value.span(),
                                    message: format!(
                                        "Invalid value for {field_name}: expected one of {}",
                                        values.join(", ")
                                    ),
                                }],
                            });
                        }
                    }
                    StringVerifier::Function(verifier) => verifier(
                        value,
                        VerificationContext {
                            field_name,
                            field_value,
                        },
                    )?,
                    StringVerifier::AcceptAny => (),
                }

                Ok(())
            }
            ast::Literal::UInt(_) | ast::Literal::Bool(_) | ast::Literal::Color(_) => {
                Err(ValidationError::make_mismatched_field_type(
                    field_name,
                    *field_value.span(),
                    "string",
                ))
            }
        },
        CompiledFieldValue::State(_) | CompiledFieldValue::Type(_) => Err(
            ValidationError::make_mismatched_field_type(field_name, *field_value.span(), "string"),
        ),
    }
}

pub enum StringVerifier<'a> {
    PossibleValues(&'a [&'a str]),
    Function(&'a StringChecker),
    AcceptAny,
}
type StringChecker = dyn Fn(&str, VerificationContext) -> Result<(), ValidationError>;

pub struct VerificationContext<'a> {
    pub field_name: &'a str,
    pub field_value: &'a Spanned<CompiledFieldValue>,
}

impl<'a: 'static> From<&'a [&'a str]> for StringVerifier<'a> {
    fn from(value: &'a [&'a str]) -> Self {
        Self::PossibleValues(value)
    }
}

impl<'a, F> From<&'a F> for StringVerifier<'a>
where
    F: Fn(&str, VerificationContext) -> Result<(), ValidationError> + 'static,
{
    fn from(value: &'a F) -> Self {
        Self::Function(value)
    }
}

pub fn expect_uint_literal<'a, V: Into<UIntVerifier<'a>>>(
    field_name: &str,
    field_value: &Spanned<CompiledFieldValue>,
    verifier: V,
) -> Result<(), ValidationError> {
    match &**field_value {
        CompiledFieldValue::Literal(literal) => match literal {
            ast::Literal::UInt(value) => {
                match verifier.into() {
                    UIntVerifier::Range(range) => {
                        if !range.contains(value) {
                            return Err(ValidationError {
                                kind: ValidationErrorKind::InvalidFieldValue,
                                message: format!("Invalid value for {field_name}"),
                                labels: vec![DiagnosticLabel {
                                    message: format!(
                                        "Invalid value for {field_name}: expected a number within {} <= .. < {}",
                                        range.start, range.end
                                    ),
                                    span: *field_value.span(),
                                }],
                            });
                        }
                    }
                    UIntVerifier::Function(verifier) => verifier(value)?,
                    UIntVerifier::AcceptAny => (),
                }

                Ok(())
            }
            ast::Literal::String(_) | ast::Literal::Bool(_) | ast::Literal::Color(_) => {
                Err(ValidationError::make_mismatched_field_type(
                    field_name,
                    *field_value.span(),
                    "uint",
                ))
            }
        },
        CompiledFieldValue::State(_) | CompiledFieldValue::Type(_) => Err(
            ValidationError::make_mismatched_field_type(field_name, *field_value.span(), "uint"),
        ),
    }
}

pub enum UIntVerifier<'a> {
    Range(std::ops::Range<u64>),
    Function(&'a UIntChecker),
    AcceptAny,
}
type UIntChecker = dyn Fn(&u64) -> Result<(), ValidationError>;

impl<'a> From<std::ops::Range<u64>> for UIntVerifier<'a> {
    fn from(value: std::ops::Range<u64>) -> Self {
        Self::Range(value)
    }
}

impl<'a: 'static> From<&'a UIntChecker> for UIntVerifier<'a> {
    fn from(value: &'a UIntChecker) -> Self {
        Self::Function(value)
    }
}

pub fn expect_bool_literal(
    field_name: &str,
    field_value: &Spanned<CompiledFieldValue>,
) -> Result<(), ValidationError> {
    match &**field_value {
        CompiledFieldValue::Literal(literal) => match literal {
            ast::Literal::Bool(_) => Ok(()),
            ast::Literal::String(_) | ast::Literal::UInt(_) | ast::Literal::Color(_) => {
                Err(ValidationError::make_mismatched_field_type(
                    field_name,
                    *field_value.span(),
                    "bool",
                ))
            }
        },
        CompiledFieldValue::State(_) | CompiledFieldValue::Type(_) => Err(
            ValidationError::make_mismatched_field_type(field_name, *field_value.span(), "bool"),
        ),
    }
}

pub fn expect_color_literal(
    field_name: &str,
    field_value: &Spanned<CompiledFieldValue>,
) -> Result<(), ValidationError> {
    match &**field_value {
        CompiledFieldValue::Literal(literal) => match literal {
            ast::Literal::Color(_) => Ok(()),
            ast::Literal::String(_) | ast::Literal::UInt(_) | ast::Literal::Bool(_) => {
                Err(ValidationError::make_mismatched_field_type(
                    field_name,
                    *field_value.span(),
                    "color",
                ))
            }
        },
        CompiledFieldValue::State(_) | CompiledFieldValue::Type(_) => Err(
            ValidationError::make_mismatched_field_type(field_name, *field_value.span(), "color"),
        ),
    }
}

pub fn expect_type(
    field_name: &str,
    field_value: &Spanned<CompiledFieldValue>,
    type_path: ast::TypePath<String>,
) -> Result<(), ValidationError> {
    match &**field_value {
        CompiledFieldValue::Type(compiled_type_declaration) => {
            if compiled_type_declaration.type_path.unspan_cloned() != type_path {
                return Err(ValidationError::make_mismatched_field_type(
                    field_name,
                    *field_value.span(),
                    &type_path.to_string(),
                ));
            }

            Ok(())
        }
        CompiledFieldValue::State(_) | CompiledFieldValue::Literal(_) => {
            Err(ValidationError::make_mismatched_field_type(
                field_name,
                *field_value.span(),
                &type_path.to_string(),
            ))
        }
    }
}

pub fn expect_state_with<F>(
    field_name: &str,
    field_value: &Spanned<CompiledFieldValue>,
    type_checker: F,
) -> Result<(), ValidationError>
where
    F: FnOnce(&ast::Literal) -> Result<(), ValidationError>,
{
    match &**field_value {
        CompiledFieldValue::State(state_bind) => type_checker(&state_bind.value),
        CompiledFieldValue::Literal(_) | CompiledFieldValue::Type(_) => Err(
            ValidationError::make_mismatched_field_type(field_name, *field_value.span(), "state"),
        ),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::analysis::*;
    use crate::parser;

    #[test]
    fn basic_validation() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();
        let mut context = SemanticAnalyzerContext::new(&build_context);

        assert!(
            parser
                .parse(
                    &mut vec![], 
                    "state SummaryText = \"some text\";

                    alias Summary = Text(value = SummaryText, font = Font(size = 14)) {};

                    FlexBox(direction = \"horizontal\") {
                        state BodyText = \"description\";
                        Box() {
                            Summary() {}

                            Text(value = BodyText, wrap = true) {}
                        }
                    }"
                )
                .unwrap()
                .analyze(&mut context, SemanticScope::default())
                .unwrap()
                .validate(&context)
                .is_ok()
        );
    }

    #[test]
    fn required_fields_are_filled() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();
        let mut context = SemanticAnalyzerContext::new(&build_context);

        assert!(
            parser
                .parse(&mut vec![], "FlexBox() {}")
                .unwrap()
                .analyze(&mut context, SemanticScope::default())
                .unwrap()
                .validate(&context)
                .is_err_and(|err| err.kind == ValidationErrorKind::MissingFields)
        );
    }

    #[test]
    fn unknown_type_in_tree() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();
        let mut context = SemanticAnalyzerContext::new(&build_context);

        assert!(
            parser
                .parse(&mut vec![], "Unknown() {}")
                .unwrap()
                .analyze(&mut context, SemanticScope::default())
                .unwrap()
                .validate(&context)
                .is_err_and(|err| err.kind == ValidationErrorKind::MissingBuilderImplementation)
        );
    }
}
