use std::collections::HashMap;

use crate::{
    ast::*,
    plugin::{BuildContext, ValidationError},
};

// Rules:
// 1. States are declared in different scopes, but they're managed globally regardless which lifetime they have.
// 2. Scopes just gives state identity and makes easy to find where the value actually assigned.
// 3. Lifetime of a state is bound to runtime but to a DSL input. And it may persist if its path is the same (even if it may cause undesired persistence between DSL inputs) or keys (in future, but not for now).
//
//
// What about alias templates? Explicit rules:
//
// 1. If a widget is aliased with a scope, then the scope is evaluated in place. A declared state within this scope is not usable after use.
// ```
// alias SomeWidget = Box() { state body = "text"; Text(value = body) {} };
//
// FlexBox() {
//   SomeWidget() {
//     Text(value = body) {} // CAUSES AN ERROR
//   }
// }
// ```
//
// It's the hygienic of widget instantiation and alias declaration. The same rule applies to aliases.
//
// 2. It means that a widget declaration may have an initial scope and subsequent scopes.
// ```
// alias A = B() { state b = 3; };
// alias C = A() {
//   // state b is not accessible
//   state c = "hello";
//   alias Summary = Text();
// };
//
// FlexBox() {
//   C() {
//     // states b and c are not accessible
//     // alias Summary is not accessible
//   }
// }
// ```

pub(crate) struct SemanticAnalyzerContext<'a> {
    table: InformationTable,
    state_bind_id_generator: u64,
    build_context: &'a BuildContext,
}

impl<'a> SemanticAnalyzerContext<'a> {
    pub(crate) fn new(build_context: &'a BuildContext) -> Self {
        Self {
            build_context,
            table: InformationTable::default(),
            state_bind_id_generator: 0,
        }
    }

    fn fork(&self) -> Self {
        Self {
            table: InformationTable::default(),
            state_bind_id_generator: self.state_bind_id_generator,
            build_context: self.build_context,
        }
    }

    fn next_state_bind_id(&mut self) -> StateBindId {
        let bind_id = StateBindId(self.state_bind_id_generator);
        self.state_bind_id_generator += 1;
        bind_id
    }
}

#[derive(Clone, Default)]
pub struct InformationTable {
    state_binds: HashMap<StateBindId, DeclarationInfo<StateDeclaration>>,
}

impl InformationTable {
    fn extend_with_path(&mut self, path: &[ScopeIdentity], subtable: Self) {
        for (state_bind, mut declaration_info) in subtable.state_binds {
            let mut full_path = path.to_owned();
            full_path.extend(declaration_info.path);
            declaration_info.path = full_path;

            self.state_binds.insert(state_bind, declaration_info);
        }
    }

    fn remap_state_bind_id(
        &mut self,
        context: &mut SemanticAnalyzerContext,
    ) -> HashMap<StateBindId, StateBindId> {
        let mut mapper = HashMap::new();

        for old_bind_id in self.state_binds.keys().copied().collect::<Vec<_>>() {
            let new_bind_id = context.next_state_bind_id();

            let declaration_info = self.state_binds.remove(&old_bind_id).unwrap();

            self.state_binds.insert(new_bind_id, declaration_info);
            mapper.insert(old_bind_id, new_bind_id);
        }

        mapper
    }

    fn state_by_bind_id(&self, bind_id: &StateBindId) -> Option<&StateDeclaration> {
        self.state_binds.get(bind_id).map(|info| &info.value)
    }
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq)]
struct StateBindId(u64);

#[derive(Default, Clone)]
pub(crate) struct SemanticScope {
    path: Vec<ScopeIdentity>,
    aliases: HashMap<String, Spanned<CompiledAliasReplacement>>,
    states: HashMap<String, StateBind>,
}

impl SemanticScope {
    fn new_global() -> Self {
        Self {
            path: vec![ScopeIdentity {
                position: 0,
                kind: ScopeIdentityKind::Global,
            }],
            ..Default::default()
        }
    }

    fn with_default_aliases(self) -> Self {
        // TODO: write here very common aliases to existing widgets
        todo!()
    }

    fn extend_with(&self, scope_identity: ScopeIdentity) -> Self {
        let mut new_scope = self.clone();
        new_scope.path.push(scope_identity);
        new_scope
    }

    fn take_path(&mut self) -> Vec<ScopeIdentity> {
        std::mem::take(&mut self.path)
    }

    fn use_path(&mut self, path: Vec<ScopeIdentity>) {
        self.path = path;
    }

    fn resolve_widget(
        &self,
        widget_declaration: &Spanned<WidgetDeclaration>,
        context: &mut SemanticAnalyzerContext,
    ) -> Result<Spanned<CompiledWidgetDeclaration>, AnalysisError> {
        let mut subtable = InformationTable::default();
        let mut children: Vec<BuildScope> = vec![];

        let compiled_widget_declaration = if widget_declaration.type_path.0.len() == 1
            && let Some(other_replacement) = self.aliases.get(&*widget_declaration.type_path.0[0])
        {
            let mut compiled_type_declaration = match &**other_replacement {
                CompiledAliasReplacement::Path(type_path) => {
                    CompiledTypeDeclaration::new(type_path.clone())
                }
                CompiledAliasReplacement::Type(compiled_type_declaration) => {
                    compiled_type_declaration.clone()
                }
                CompiledAliasReplacement::Widget(CompiledWidgetDeclaration {
                    subtable: inner_subtable,
                    scope,
                }) => {
                    subtable = inner_subtable.clone();
                    children = scope.children.clone();

                    let BuildScopeKind::Widget(compiled_type_declaration) = &scope.kind else {
                        return Err(AnalysisError {
                            kind: AnalysisErrorKind::InvalidScopeKind,
                            message: "The input is not valid or parser faield to ensure correctness: a scope is not matched to a widget".to_owned(),
                            labels: vec![
                                DiagnosticLabel {
                                    message: "A widget is declared here".to_owned(),
                                    span: *widget_declaration.span()
                                },
                                DiagnosticLabel {
                                    message: "An alias replacement is declared here".to_owned(),
                                    span: *other_replacement.span()
                                }
                            ]
                        });
                    };

                    compiled_type_declaration.clone()
                }
            };

            let widget_declaration =
                CompiledTypeDeclaration::from_widget_declaration(widget_declaration, self)?;

            compiled_type_declaration.extend_fields(&widget_declaration);

            let mut compiled_widget_declaration = CompiledWidgetDeclaration {
                subtable,
                scope: BuildScope {
                    kind: BuildScopeKind::Widget(compiled_type_declaration),
                    children,
                },
            };
            compiled_widget_declaration.remap_state_bind_id(context)?;

            compiled_widget_declaration
        } else {
            CompiledWidgetDeclaration {
                subtable,
                scope: BuildScope {
                    kind: BuildScopeKind::Widget(CompiledTypeDeclaration::from_widget_declaration(
                        widget_declaration,
                        self,
                    )?),
                    children,
                },
            }
        };

        Ok(Spanned::new(
            compiled_widget_declaration,
            widget_declaration.span().start,
            widget_declaration.span().end,
        ))
    }

    fn resolve_type(
        &self,
        type_declaration: &Spanned<TypeDeclaration>,
    ) -> Result<Spanned<CompiledTypeDeclaration>, AnalysisError> {
        let compiled_type_declaration = if type_declaration.type_path.0.len() == 1
            && let Some(other_replacement) = self.aliases.get(&*type_declaration.type_path.0[0])
        {
            let mut aliased_type_declaration = match &**other_replacement {
                CompiledAliasReplacement::Path(type_path) => {
                    CompiledTypeDeclaration::new(type_path.clone())
                }
                CompiledAliasReplacement::Type(compiled_type_declaration) => {
                    compiled_type_declaration.clone()
                }
                CompiledAliasReplacement::Widget(_) => {
                    return Err(AnalysisError {
                        kind: AnalysisErrorKind::IncompatibleType,
                        message: "A widget cannot be used as a type".to_owned(),
                        labels: vec![
                            DiagnosticLabel {
                                message: "A type is declared here".to_owned(),
                                span: *type_declaration.span(),
                            },
                            DiagnosticLabel {
                                message: "An alias replacement is declared here".to_owned(),
                                span: *other_replacement.span(),
                            },
                        ],
                    });
                }
            };

            let compiled_type_declaration =
                CompiledTypeDeclaration::from_type_declaration(type_declaration, self)?;

            for (name, value) in compiled_type_declaration.fields {
                aliased_type_declaration.fields.insert(name, value);
            }

            aliased_type_declaration
        } else {
            CompiledTypeDeclaration::from_type_declaration(type_declaration, self)?
        };

        Ok(Spanned::new(
            compiled_type_declaration,
            type_declaration.span().start,
            type_declaration.span().end,
        ))
    }

    fn resolve_type_path(
        &self,
        type_path: &Spanned<TypePath<Spanned<String>>>,
    ) -> Result<Spanned<CompiledAliasReplacement>, AnalysisError> {
        Ok(
            if type_path.0.len() == 1
                && let Some(other_replacement) = self.aliases.get(&*type_path.0[0])
            {
                other_replacement.clone().relocate(*type_path.span())
            } else {
                Spanned::new(
                    CompiledAliasReplacement::Path(type_path.clone()),
                    type_path.span().start,
                    type_path.span().end,
                )
            },
        )
    }

    fn resolve_field_value(
        &self,
        field_value: &Spanned<FieldValue>,
    ) -> Result<Spanned<CompiledFieldValue>, AnalysisError> {
        let field_value_span = field_value.span();

        Ok(Spanned::new(
            match &**field_value {
                FieldValue::State(name) => CompiledFieldValue::State(
                    self.states
                        .get(name)
                        .ok_or(AnalysisError {
                            kind: AnalysisErrorKind::MissingState,
                            message: "A state is missing to bind to a widget".to_owned(),
                            labels: vec![DiagnosticLabel {
                                message: "The bind happens here".to_owned(),
                                span: *field_value.span(),
                            }],
                        })?
                        .clone(),
                ),
                FieldValue::Literal(literal) => CompiledFieldValue::Literal(literal.clone()),
                FieldValue::Type(type_declaration) => CompiledFieldValue::Type(
                    self.resolve_type(&Spanned::new(
                        type_declaration.clone(),
                        field_value_span.start,
                        field_value_span.end,
                    ))?
                    .unspan(),
                ),
            },
            field_value_span.start,
            field_value_span.end,
        ))
    }

    fn append_alias(
        &mut self,
        alias: &AliasDeclaration,
        context: &SemanticAnalyzerContext,
    ) -> Result<(), AnalysisError> {
        fn is_name_duplicates(alias_name: &str, type_path: &TypePath<Spanned<String>>) -> bool {
            type_path.0.len() == 1 && *type_path.0[0] == alias_name
        }

        fn duplicate_name_error<T>(
            alias_name: &Spanned<String>,
            replacement_type_path: &Spanned<T>,
        ) -> AnalysisError {
            AnalysisError {
                kind: AnalysisErrorKind::InvalidAliasDeclaration,
                message: "Declaring an alias with the same name as type is not permitted"
                    .to_owned(),
                labels: vec![
                    DiagnosticLabel {
                        span: *alias_name.span(),
                        message: "The alias name is declared here".to_owned(),
                    },
                    DiagnosticLabel {
                        span: *replacement_type_path.span(),
                        message: "The type name is declared here".to_owned(),
                    },
                ],
            }
        }

        let compiled_replacement = match &*alias.replacement {
            AliasReplacement::Widget(scope_declaration) => {
                let ScopeDeclaration {
                    kind: ScopeKind::Widget(widget_declaration),
                    ..
                } = scope_declaration
                else {
                    return Err(AnalysisError {
                        kind: AnalysisErrorKind::InvalidScopeKind,
                        message: "The input is not valid or parser faield to ensure correctness: a scope is not matched to a widget".to_owned(),
                        labels: vec![DiagnosticLabel{
                            span: *alias.replacement.span(),
                            message: "A scope is not valid".to_owned(),
                        }]
                    });
                };

                if is_name_duplicates(&alias.name, &widget_declaration.type_path) {
                    return Err(duplicate_name_error(
                        &alias.name,
                        &widget_declaration.type_path,
                    ));
                }

                let mut context = context.fork();
                let current_path = self.take_path();

                let build_scope = scope_declaration.analyze(&mut context, self.clone());
                self.use_path(current_path);

                Spanned::new(
                    CompiledAliasReplacement::Widget(CompiledWidgetDeclaration {
                        subtable: context.table,
                        scope: build_scope?,
                    }),
                    alias.replacement.span().start,
                    alias.replacement.span().end,
                )
            }
            AliasReplacement::Type(type_declaration) => {
                if is_name_duplicates(&alias.name, &type_declaration.type_path) {
                    return Err(duplicate_name_error(
                        &alias.name,
                        &type_declaration.type_path,
                    ));
                }

                self.resolve_type(&Spanned::new(
                    type_declaration.clone(),
                    alias.replacement.span().start,
                    alias.replacement.span().end,
                ))?
                .map(CompiledAliasReplacement::Type)
            }
            AliasReplacement::Path(type_path) => {
                if is_name_duplicates(&alias.name, type_path) {
                    return Err(duplicate_name_error(&alias.name, &alias.replacement));
                }

                let span = alias.replacement.span();
                self.resolve_type_path(&Spanned::new(type_path.clone(), span.start, span.end))?
            }
        };

        self.aliases
            .insert((*alias.name).clone(), compiled_replacement);

        Ok(())
    }

    fn append_state(
        &mut self,
        state: &StateDeclaration,
        position_before_next_scope: usize,
        context: &mut SemanticAnalyzerContext,
    ) {
        let bind_id = context.next_state_bind_id();

        context.table.state_binds.insert(
            bind_id,
            DeclarationInfo {
                path: self.path.clone(),
                position_before_next_scope,
                value: state.clone(),
            },
        );

        self.states.insert(
            (*state.name).clone(),
            StateBind {
                bind_id,
                value: (*state.value).clone(),
            },
        );
    }
}

#[derive(Clone)]
struct DeclarationInfo<T> {
    path: Vec<ScopeIdentity>,
    position_before_next_scope: usize,
    value: T,
}

/// Defines the identity of a scope.
///
/// To identify an arbitrary scope uses its position between its siblings and kind of this scope.
/// The identity is meaningful only wihtin a particular scope but outside of it.
///
/// Therefore, to correctly locate a scope, need to build path containing scope identities.
#[derive(Debug, Clone, PartialEq)]
struct ScopeIdentity {
    position: usize,
    kind: ScopeIdentityKind,
}

#[derive(Debug, Clone, PartialEq)]
enum ScopeIdentityKind {
    Global,
    Widget(TypePath<String>),
}

pub(crate) trait SemanticAnalyzer {
    type Output;

    fn analyze(
        &self,
        context: &mut SemanticAnalyzerContext,
        scope: SemanticScope,
    ) -> Result<Self::Output, AnalysisError>;
}

#[derive(Debug, PartialEq)]
pub(crate) struct AnalysisError {
    kind: AnalysisErrorKind,
    message: String,
    labels: Vec<DiagnosticLabel>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum AnalysisErrorKind {
    InvalidScopeKind,
    InvalidAliasDeclaration,
    IncompatibleType,
    DuplicateFieldName,
    MissingState,
    NoWidgetAtGlobalScope,
    ManyWidgetsAtGlobalScope,
}

#[derive(Debug, PartialEq)]
pub(crate) struct DiagnosticLabel {
    pub(crate) span: Span,
    pub(crate) message: String,
}

impl SemanticAnalyzer for ScopeDeclaration {
    type Output = BuildScope;

    fn analyze(
        &self,
        context: &mut SemanticAnalyzerContext,
        mut scope: SemanticScope,
    ) -> Result<Self::Output, AnalysisError> {
        let mut children = vec![];

        let kind = match &self.kind {
            ScopeKind::Global => {
                let total_widgets = self
                    .children
                    .iter()
                    .filter(|&child| matches!(&**child, InScopeDeclaration::Widget(_)))
                    .collect::<Vec<_>>();

                if total_widgets.is_empty() {
                    return Err(AnalysisError {
                        kind: AnalysisErrorKind::NoWidgetAtGlobalScope,
                        message: "Expected at least one Widget at the global scope".to_owned(),
                        labels: vec![],
                    });
                }

                if total_widgets.len() > 1 {
                    return Err(AnalysisError {
                        kind: AnalysisErrorKind::ManyWidgetsAtGlobalScope,
                        message: "Multiple widgets at the global scope is not permitted".to_owned(),
                        labels: total_widgets
                            .iter()
                            .map(|child| DiagnosticLabel {
                                span: *child.span(),
                                message: "A widget is declared here".to_owned(),
                            })
                            .collect(),
                    });
                }

                BuildScopeKind::Global
            }
            ScopeKind::Widget(widget_declaration) => {
                let compiled_widget_declaration =
                    scope.resolve_widget(widget_declaration, context)?.unspan();

                context
                    .table
                    .extend_with_path(&scope.path, compiled_widget_declaration.subtable);

                children = compiled_widget_declaration.scope.children;
                compiled_widget_declaration.scope.kind
            }
        };

        let mut scope_position = children.len();

        for child in &self.children {
            match &**child {
                InScopeDeclaration::Alias(alias_declaration) => {
                    scope.append_alias(alias_declaration, context)?;
                }
                InScopeDeclaration::State(state_declaration) => {
                    scope.append_state(state_declaration, scope_position, context);
                }
                InScopeDeclaration::Widget(scope_declaration) => {
                    let ScopeKind::Widget(widget) = &scope_declaration.kind else {
                        return Err(AnalysisError {
                            kind: AnalysisErrorKind::InvalidScopeKind,
                            message: "The input is not valid or parser faield to ensure correctness: a scope is not matched to a widget".to_owned(),
                            labels: vec![DiagnosticLabel{
                                span: *child.span(),
                                message: "A scope is not valid".to_owned(),
                            }]
                        });
                    };

                    children.push(scope_declaration.analyze(
                        context,
                        scope.extend_with(ScopeIdentity {
                            position: scope_position,
                            kind: ScopeIdentityKind::Widget(widget.type_path.unspan_cloned()),
                        }),
                    )?);

                    scope_position += 1;
                }
            }
        }

        Ok(BuildScope { kind, children })
    }
}

#[derive(Clone)]
pub(crate) struct BuildScope {
    kind: BuildScopeKind,
    children: Vec<BuildScope>,
}

impl BuildScope {
    fn remap_state_bind_id(
        &mut self,
        mapper: &HashMap<StateBindId, StateBindId>,
    ) -> Result<(), AnalysisError> {
        match &mut self.kind {
            BuildScopeKind::Global => (),
            BuildScopeKind::Widget(compiled_type_declaration) => {
                compiled_type_declaration.remap_state_bind_id(mapper)?;
            }
        }

        for scope in &mut self.children {
            scope.remap_state_bind_id(mapper)?
        }

        Ok(())
    }
}

#[derive(Clone)]
pub(crate) enum BuildScopeKind {
    Global,
    Widget(CompiledTypeDeclaration),
}

/// Represents the alias replacement that is already prepared to use in place.
///
/// Original AST is converted to build-compatible types, making the build process more easy.
#[derive(Clone)]
enum CompiledAliasReplacement {
    Path(Spanned<TypePath<Spanned<String>>>),
    Type(CompiledTypeDeclaration),
    Widget(CompiledWidgetDeclaration),
}

/// Represents the ready-to-build widget information with its scope.
///
/// The subtable contains important information about path to particular declarations.
/// And to merge it to existing one, you have to know that subtable contains only traling path.
/// The table below will help you to understand.
///
///
/// | global/WidgetA/WidgetB/ |      {PLACEHOLDER}     |  WidgetC/State |
/// |:-----------------------:|:----------------------:|:--------------:|
/// |        ^                |           ^            |       ^        |
/// |    prefix path          | usually an alias scope |  suffix path   |
///
/// Usually if a current widget is replaced by an alias replacement, then its subtable rebases on
/// current one. Let's name a current widget as "WidgetD", then the path looks like that
///
/// `global/WidgetA/WidgetB/WidgetD`
///
/// While a path in subtable looks like that
///
/// `WidgetC/State`
///
/// The resulting path must look like that
///
/// `global/WidgetA/WidgetB/WidgetD/WidgetC/State`.
#[derive(Clone)]
struct CompiledWidgetDeclaration {
    subtable: InformationTable,
    scope: BuildScope,
}

impl CompiledWidgetDeclaration {
    fn remap_state_bind_id(
        &mut self,
        context: &mut SemanticAnalyzerContext,
    ) -> Result<(), AnalysisError> {
        let mapper = self.subtable.remap_state_bind_id(context);
        self.scope.remap_state_bind_id(&mapper)
    }
}

#[derive(Clone)]
pub struct CompiledTypeDeclaration {
    pub type_path: Spanned<TypePath<Spanned<String>>>,
    pub fields: HashMap<String, Spanned<CompiledFieldValue>>,
    field_spans: HashMap<String, Span>,
}

impl CompiledTypeDeclaration {
    fn new(type_path: Spanned<TypePath<Spanned<String>>>) -> Self {
        Self {
            type_path,
            fields: HashMap::new(),
            field_spans: HashMap::new(),
        }
    }

    fn from_widget_declaration(
        widget_declaration: &WidgetDeclaration,
        scope: &SemanticScope,
    ) -> Result<Self, AnalysisError> {
        let mut fields = HashMap::new();
        let mut field_spans = HashMap::new();

        for field in &widget_declaration.fields {
            let field_name = field.name.clone_value();
            let last_inserted =
                fields.insert(field_name.clone(), scope.resolve_field_value(&field.value)?);

            if last_inserted.is_some() {
                return Err(AnalysisError {
                    kind: AnalysisErrorKind::DuplicateFieldName,
                    message: "The fields are duplicated".to_owned(),
                    labels: vec![
                        DiagnosticLabel {
                            span: *field_spans
                                .get(&field_name)
                                .expect("Field span is registered with itself"),
                            message: "A first field declared here".to_owned(),
                        },
                        DiagnosticLabel {
                            span: *field.span(),
                            message: "A second field declared here".to_owned(),
                        },
                    ],
                });
            }

            field_spans.insert(field_name, *field.span());
        }

        Ok(Self {
            type_path: widget_declaration.type_path.clone(),
            fields,
            field_spans,
        })
    }

    fn from_type_declaration(
        type_declaration: &TypeDeclaration,
        scope: &SemanticScope,
    ) -> Result<Self, AnalysisError> {
        let mut fields = HashMap::new();
        let mut field_spans = HashMap::new();

        for field in &type_declaration.fields {
            let field_name = field.name.clone_value();
            let last_inserted =
                fields.insert(field_name.clone(), scope.resolve_field_value(&field.value)?);

            if last_inserted.is_some() {
                return Err(AnalysisError {
                    kind: AnalysisErrorKind::DuplicateFieldName,
                    message: "The fields are duplicated".to_owned(),
                    labels: vec![
                        DiagnosticLabel {
                            span: *field_spans
                                .get(&field_name)
                                .expect("Field span is registered with itself"),
                            message: "A first field declared here".to_owned(),
                        },
                        DiagnosticLabel {
                            span: *field.span(),
                            message: "A second field declared here".to_owned(),
                        },
                    ],
                });
            }

            field_spans.insert(field_name, *field.span());
        }

        Ok(Self {
            type_path: type_declaration.type_path.clone(),
            fields,
            field_spans,
        })
    }

    pub(crate) fn get_field_span(&self, field_name: &str) -> Option<&Span> {
        self.field_spans.get(field_name)
    }

    fn remap_state_bind_id(
        &mut self,
        mapper: &HashMap<StateBindId, StateBindId>,
    ) -> Result<(), AnalysisError> {
        for value in self.fields.values_mut() {
            value.remap_state_bind_id(mapper)?;
        }

        Ok(())
    }

    fn extend_fields(&mut self, other: &Self) {
        for (name, value) in &other.fields {
            self.fields.insert(name.clone(), value.clone());
        }

        for (name, span) in &other.field_spans {
            self.field_spans.insert(name.clone(), *span);
        }
    }
}

#[derive(Clone)]
pub enum CompiledFieldValue {
    State(StateBind),
    Literal(Literal),
    Type(CompiledTypeDeclaration),
}

impl CompiledFieldValue {
    fn remap_state_bind_id(
        &mut self,
        mapper: &HashMap<StateBindId, StateBindId>,
    ) -> Result<(), AnalysisError> {
        match self {
            CompiledFieldValue::State(state_bind) => {
                // INFO: if state is not persist in current subtable, then it's bound to state in
                // scope outside. The global state bind id maintains correctness.
                if let Some(&new_bind_id) = mapper.get(&state_bind.bind_id) {
                    state_bind.bind_id = new_bind_id;
                }
            }
            CompiledFieldValue::Literal(_) => (),
            CompiledFieldValue::Type(compiled_type_declaration) => {
                compiled_type_declaration.remap_state_bind_id(mapper)?
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct StateBind {
    bind_id: StateBindId,
    pub value: Literal,
}

pub(crate) trait Validate {
    fn validate(&self, context: &SemanticAnalyzerContext) -> Result<(), ValidationError>;
}

impl Validate for BuildScope {
    fn validate(&self, context: &SemanticAnalyzerContext) -> Result<(), ValidationError> {
        match &self.kind {
            BuildScopeKind::Global => (),
            BuildScopeKind::Widget(compiled_type_declaration) => {
                let Some(builder) = context
                    .build_context
                    .builder_by_type_path(&compiled_type_declaration.type_path.to_string())
                else {
                    return Err(ValidationError::make_missing_builder_implementation(
                        compiled_type_declaration,
                    ));
                };

                builder.validate(compiled_type_declaration, context.build_context)?;
            }
        }

        for scope in &self.children {
            scope.validate(context)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ast, parser};

    #[test]
    fn root_widget_singularity() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();

        assert!(
            parser
                .parse(&mut vec![], "alias SomeWidget = Box() {};",)
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_err_and(|err| err.kind == AnalysisErrorKind::NoWidgetAtGlobalScope)
        );

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "FlexBox(direction = \"horizontal\") {}
                    
                    Text() {}",
                )
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_err_and(
                    |err| err.kind == AnalysisErrorKind::ManyWidgetsAtGlobalScope
                        && err
                            .labels
                            .iter()
                            .all(|label| label.span == Span { start: 0, end: 36 }
                                || label.span == Span { start: 78, end: 87 })
                )
        );
    }

    #[test]
    fn basic_syntax_and_nested_type_declarations() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "FlexBox(
                        direction = \"horizontal\"
                    ) {
                        state body_state = \"text\";

                        Text(value = body_state) {}
                    }",
                )
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_ok()
        );

        assert!(
            parser
                .parse(&mut vec![], "FlexBox(border = Border(size = 3)) {}")
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_ok()
        );

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "FlexBox(
                        border = Border(
                            size = 3,
                            color = #FF0000
                        )
                    ) {}",
                )
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_ok()
        );
    }

    #[test]
    fn state_declarations_and_references() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "FlexBox() {
                        state text = \"hello\";
                        Text(value = text) {}
                    }",
                )
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_ok()
        );

        assert!(
            parser
                .parse(&mut vec![], "Text(value = some_state) {}",)
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_err_and(|err| err.kind == AnalysisErrorKind::MissingState
                    && err.labels.len() == 1
                    && err.labels.first().unwrap().span == Span { start: 13, end: 23 })
        );
    }

    #[test]
    fn alias_local_state_and_lexical_hygiene() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "alias TextRow = FlexBox(direction = \"horizontal\") {
                        state body_state = \"value\";
                        Text(value = body_state) {}
                    };

                    FlexBox(direction = \"vertical\") {
                        TextRow() {}
                    }",
                )
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_ok()
        );

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "alias TextRow = FlexBox(direction = \"horizontal\") {
                        Text(value = body_state) {}
                    };

                    FlexBox(direction = \"vertical\") {
                        state body_state = \"value\";
                        TextRow() {}
                    }",
                )
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_err_and(|err| err.kind == AnalysisErrorKind::MissingState
                    && err.labels.len() == 1
                    && err.labels.first().unwrap().span == Span { start: 89, end: 99 })
        );

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "alias SomeWidget = FlexBox(direction = \"horizontal\") {
                        state body = \"text\";
                        Text(value = body) {}
                    };

                    FlexBox(direction = \"vertical\") {
                        SomeWidget() {
                            Text(value = body) {}
                        }
                    }",
                )
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_err_and(|err| err.kind == AnalysisErrorKind::MissingState
                    && err.labels.len() == 1
                    && err.labels.first().unwrap().span
                        == Span {
                            start: 304,
                            end: 308
                        })
        );

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "alias Container = Box() {
                        state internal_value = \"internal\";
                        Text(value = internal_value) {}
                    };

                    FlexBox() {
                        state external = \"external\";
                        Container() {
                            Text(value = external) {}
                        }
                    }",
                )
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_ok()
        );
    }

    #[test]
    fn repeated_alias_instantiation_and_state_identity() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();
        let mut semantic_analyzer = SemanticAnalyzerContext::new(&build_context);

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "alias BasicText = Box() {
                        state value = \"some_value\";
                        Text(value = value) {}
                    };

                    FlexBox() {
                        BasicText() {}
                        BasicText() {}
                    }",
                )
                .unwrap()
                .analyze(&mut semantic_analyzer, SemanticScope::new_global())
                .is_ok()
        );

        assert!(semantic_analyzer.table.state_binds.values().all(|info| {
            (info.path
                == vec![
                    ScopeIdentity {
                        position: 0,
                        kind: ScopeIdentityKind::Global,
                    },
                    ScopeIdentity {
                        position: 0,
                        kind: ScopeIdentityKind::Widget(TypePath(vec!["FlexBox".to_owned()])),
                    },
                    ScopeIdentity {
                        position: 0,
                        kind: ScopeIdentityKind::Widget(TypePath(vec!["BasicText".to_owned()])),
                    },
                ]
                || info.path
                    == vec![
                        ScopeIdentity {
                            position: 0,
                            kind: ScopeIdentityKind::Global,
                        },
                        ScopeIdentity {
                            position: 0,
                            kind: ScopeIdentityKind::Widget(TypePath(vec!["FlexBox".to_owned()])),
                        },
                        ScopeIdentity {
                            position: 1,
                            kind: ScopeIdentityKind::Widget(TypePath(vec!["BasicText".to_owned()])),
                        },
                    ])
                && info.position_before_next_scope == 0
        }));
    }

    #[test]
    fn chained_aliases_and_overrides() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "alias Row = FlexBox(direction = \"horizontal\") {};

                    alias PrettyRow = Row(direction = \"vertical\") {};

                    PrettyRow() {}",
                )
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_ok()
        );

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "alias TickBorder = Border(size = 8);

                    alias TickRoundBorder = TickBorder(size = 10, radius = 5);

                    FlexBox(border = TickRoundBorder()) {}",
                )
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_ok()
        );
    }

    #[test]
    fn alias_template_hygiene_across_multiple_levels() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "alias Inner = Box() {
                        state b = \"inner\";
                        Text(value = b) {}
                    };

                    alias Outer = Inner() {
                        state c = \"outer\";
                        Text(value = c) {}
                    };

                    FlexBox() {
                        Outer() {
                            Text(value = b) {}
                            Text(value = c) {}
                        }
                    }",
                )
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_err_and(|err| err.kind == AnalysisErrorKind::MissingState
                    && err.labels.len() == 1
                    && err.labels.first().unwrap().span
                        == Span {
                            start: 393,
                            end: 394
                        })
        );
    }

    #[test]
    fn duplicate_fields_and_shadowing() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "state first = \"first\";
                    state second = \"second\";
                    Text(value = first, value = second) {}",
                )
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_err_and(|err| err.kind == AnalysisErrorKind::DuplicateFieldName
                    && err.labels.len() == 2
                    && err.labels.iter().all(|label| label.span
                        == Span {
                            start: 93,
                            end: 106
                        }
                        || label.span
                            == Span {
                                start: 108,
                                end: 122
                            }))
        );

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "FlexBox() {
                        state value = \"outer\";

                        FlexBox() {
                            state value = \"inner\";
                            Text(value = value) {}
                        }
                    }",
                )
                .unwrap()
                .analyze(
                    &mut SemanticAnalyzerContext::new(&build_context),
                    SemanticScope::default()
                )
                .is_ok()
        );

        let mut analyzer_context = SemanticAnalyzerContext::new(&build_context);
        let build_scope = parser
            .parse(
                &mut vec![],
                "FlexBox() {
                        state value = \"first\";
                        state value = \"second\";

                        FlexBox() {
                            Text(value = value) {}
                        }
                    }",
            )
            .unwrap()
            .analyze(&mut analyzer_context, SemanticScope::new_global());

        assert!(build_scope.is_ok());
        assert_eq!(analyzer_context.table.state_binds.len(), 2);

        let build_scope = build_scope.unwrap();

        match &build_scope.children[0].children[0].children[0].kind {
            BuildScopeKind::Global => panic!("It must be a widget"),
            BuildScopeKind::Widget(compiled_type_declaration) => {
                assert_eq!(
                    compiled_type_declaration.type_path,
                    Spanned::new(
                        TypePath(vec![Spanned::new("Text".to_owned(), 172, 176)]),
                        172,
                        176
                    )
                );

                let state_field = compiled_type_declaration.fields.get("value");
                assert!(state_field.is_some());
                let state_field = state_field.unwrap();

                match &**state_field {
                    CompiledFieldValue::State(state_bind) => {
                        assert_eq!(state_bind.value, Literal::String("second".to_owned()));

                        let state_info = analyzer_context
                            .table
                            .state_binds
                            .get(&state_bind.bind_id)
                            .unwrap();
                        assert_eq!(
                            state_info.path,
                            vec![
                                ScopeIdentity {
                                    kind: ScopeIdentityKind::Global,
                                    position: 0,
                                },
                                ScopeIdentity {
                                    kind: ScopeIdentityKind::Widget(TypePath(vec![
                                        "FlexBox".to_owned()
                                    ])),
                                    position: 0,
                                },
                            ]
                        );
                        assert_eq!(state_info.position_before_next_scope, 0);
                    }
                    _ => panic!("Invalid value"),
                }
            }
        }

        let mut analyzer_context = SemanticAnalyzerContext::new(&build_context);
        let build_scope = parser
            .parse(
                &mut vec![],
                "FlexBox() {

                        FlexBox() {
                            state value = \"first\";
                            Text(value = value) {}

                            state value = \"second\";
                            Text(value = value) {}
                        }
                    }",
            )
            .unwrap()
            .analyze(&mut analyzer_context, SemanticScope::new_global());

        assert!(build_scope.is_ok());
        assert_eq!(analyzer_context.table.state_binds.len(), 2);

        let build_scope = build_scope.unwrap();

        match &build_scope.children[0].children[0].children[1].kind {
            BuildScopeKind::Global => panic!("It must be a widget"),
            BuildScopeKind::Widget(compiled_type_declaration) => {
                assert_eq!(
                    compiled_type_declaration.type_path,
                    Spanned::new(
                        TypePath(vec![Spanned::new("Text".to_owned(), 232, 236)]),
                        232,
                        236
                    )
                );

                let state_field = compiled_type_declaration.fields.get("value");
                assert!(state_field.is_some());
                let state_field = state_field.unwrap();

                match &**state_field {
                    CompiledFieldValue::State(state_bind) => {
                        assert_eq!(state_bind.value, Literal::String("second".to_owned()));

                        let state_info = analyzer_context
                            .table
                            .state_binds
                            .get(&state_bind.bind_id)
                            .unwrap();
                        assert_eq!(
                            state_info.path,
                            vec![
                                ScopeIdentity {
                                    kind: ScopeIdentityKind::Global,
                                    position: 0,
                                },
                                ScopeIdentity {
                                    kind: ScopeIdentityKind::Widget(TypePath(vec![
                                        "FlexBox".to_owned()
                                    ])),
                                    position: 0,
                                },
                                ScopeIdentity {
                                    kind: ScopeIdentityKind::Widget(TypePath(vec![
                                        "FlexBox".to_owned()
                                    ])),
                                    position: 0,
                                },
                            ]
                        );
                        assert_eq!(state_info.position_before_next_scope, 1);
                    }
                    _ => panic!("Invalid value"),
                }
            }
        }
    }

    #[test]
    fn outer_state_in_alias_replacement() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();

        let mut analyzer_context = SemanticAnalyzerContext::new(&build_context);
        let _build_scope = parser
            .parse(
                &mut vec![],
                "state SomeState = \"value\";

                alias SomeText = Text(value = SomeState) {};

                FlexBox() {
                    SomeText() {}
                }",
            )
            .unwrap()
            .analyze(&mut analyzer_context, SemanticScope::new_global())
            .unwrap();

        assert_eq!(analyzer_context.table.state_binds.len(), 1);

        let mut analyzer_context = SemanticAnalyzerContext::new(&build_context);
        let _build_scope = parser
            .parse(
                &mut vec![],
                "state SomeState = \"value\";

                alias SomeText = Text(value = SomeState) {};

                alias OtherText = SomeText(value = SomeState) {};

                FlexBox() {
                    SomeText() {}
                }",
            )
            .unwrap()
            .analyze(&mut analyzer_context, SemanticScope::new_global())
            .unwrap();

        assert_eq!(analyzer_context.table.state_binds.len(), 1);
    }

    #[test]
    fn scope_identity_and_declaration_paths() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();

        let mut context = SemanticAnalyzerContext::new(&build_context);
        assert!(
            parser
                .parse(
                    &mut vec![],
                    "FlexBox() {
                        Box() {
                            state value = \"first\";
                            Text(value = value) {}
                        }

                        Box() {
                            state value = \"second\";
                            Text(value = value) {}
                        }
                    }",
                )
                .unwrap()
                .analyze(&mut context, SemanticScope::new_global())
                .is_ok_and(|_| {
                    let valid_paths = [
                        vec![
                            ScopeIdentity {
                                position: 0,
                                kind: ScopeIdentityKind::Global,
                            },
                            ScopeIdentity {
                                position: 0,
                                kind: ScopeIdentityKind::Widget(ast::TypePath(vec![
                                    "FlexBox".to_owned(),
                                ])),
                            },
                            ScopeIdentity {
                                position: 0,
                                kind: ScopeIdentityKind::Widget(ast::TypePath(vec![
                                    "Box".to_owned(),
                                ])),
                            },
                        ],
                        vec![
                            ScopeIdentity {
                                position: 0,
                                kind: ScopeIdentityKind::Global,
                            },
                            ScopeIdentity {
                                position: 0,
                                kind: ScopeIdentityKind::Widget(ast::TypePath(vec![
                                    "FlexBox".to_owned(),
                                ])),
                            },
                            ScopeIdentity {
                                position: 1,
                                kind: ScopeIdentityKind::Widget(ast::TypePath(vec![
                                    "Box".to_owned(),
                                ])),
                            },
                        ],
                    ];

                    for info in context.table.state_binds.values() {
                        assert!(valid_paths.contains(&info.path));
                        assert_eq!(info.position_before_next_scope, 0);

                        assert!(
                            info.value.name == Spanned::new("value".to_owned(), 78, 83)
                                || info.value.name == Spanned::new("value".to_owned(), 239, 244)
                        );
                    }

                    true
                })
        );

        let mut context = SemanticAnalyzerContext::new(&build_context);
        assert!(
            parser
                .parse(
                    &mut vec![],
                    "FlexBox() {
                        Box() {
                            state value = \"outer\";
                            Text(value = value) {}

                            Box() {
                                state value = \"inner\";
                                Text(value = value) {}
                            }
                        }
                    }",
                )
                .unwrap()
                .analyze(&mut context, SemanticScope::new_global())
                .is_ok_and(|_| {
                    let valid_paths = [
                        vec![
                            ScopeIdentity {
                                position: 0,
                                kind: ScopeIdentityKind::Global,
                            },
                            ScopeIdentity {
                                position: 0,
                                kind: ScopeIdentityKind::Widget(ast::TypePath(vec![
                                    "FlexBox".to_owned(),
                                ])),
                            },
                            ScopeIdentity {
                                position: 0,
                                kind: ScopeIdentityKind::Widget(ast::TypePath(vec![
                                    "Box".to_owned(),
                                ])),
                            },
                        ],
                        vec![
                            ScopeIdentity {
                                position: 0,
                                kind: ScopeIdentityKind::Global,
                            },
                            ScopeIdentity {
                                position: 0,
                                kind: ScopeIdentityKind::Widget(ast::TypePath(vec![
                                    "FlexBox".to_owned(),
                                ])),
                            },
                            ScopeIdentity {
                                position: 0,
                                kind: ScopeIdentityKind::Widget(ast::TypePath(vec![
                                    "Box".to_owned(),
                                ])),
                            },
                            ScopeIdentity {
                                position: 1,
                                kind: ScopeIdentityKind::Widget(ast::TypePath(vec![
                                    "Box".to_owned(),
                                ])),
                            },
                        ],
                    ];

                    for info in context.table.state_binds.values() {
                        assert!(valid_paths.contains(&info.path));
                        assert_eq!(info.position_before_next_scope, 0);

                        assert!(
                            info.value.name == Spanned::new("value".to_owned(), 78, 83)
                                || info.value.name == Spanned::new("value".to_owned(), 221, 226)
                        );
                    }

                    true
                })
        );
    }
}
