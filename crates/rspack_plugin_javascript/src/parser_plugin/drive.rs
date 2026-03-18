use rspack_core::{
  CompilerOptions, JavascriptParserOptions, ModuleLayer, ModuleType, ResourceData,
};
use rspack_hook::{Hook, HookMap, Interceptor, define_hook};
use swc_core::{
  atoms::Atom,
  common::Span,
  ecma::ast::{
    AssignExpr, BinExpr, CallExpr, Callee, ClassMember, CondExpr, Expr, IfStmt, MemberExpr,
    OptChainExpr, UnaryExpr, VarDeclarator,
  },
};

use crate::{
  parser_and_generator::ParserRuntimeRequirementsData,
  parser_plugin::{BoxJavascriptParserPlugin, JavascriptParserPluginContext},
  utils::eval::BasicEvaluatedExpression,
  visitors::{
    ClassDeclOrExpr, DestructuringAssignmentProperty, ExportDefaultDeclaration,
    ExportDefaultExpression, ExportImport, ExportLocal, ExportedVariableInfo, JavascriptParser,
    Statement, VariableDeclaration,
  },
};

define_hook!(JavascriptParserPreStatement: SyncBail(parser: &mut JavascriptParser, stmt: Statement) -> bool);
define_hook!(JavascriptParserBlockPreStatement: SyncBail(parser: &mut JavascriptParser, stmt: Statement) -> bool);
define_hook!(JavascriptParserTopLevelAwaitExpr: Sync(parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::AwaitExpr));
define_hook!(JavascriptParserTopLevelForOfAwaitStmt: Sync(parser: &mut JavascriptParser, stmt: &swc_core::ecma::ast::ForOfStmt));
define_hook!(JavascriptParserCanRename: SyncBail(parser: &mut JavascriptParser, name: &str) -> bool);
define_hook!(JavascriptParserRename: SyncBail(parser: &mut JavascriptParser, expr: &Expr, name: &str) -> bool);
define_hook!(JavascriptParserProgram: SyncBail(parser: &mut JavascriptParser, ast: &swc_core::ecma::ast::Program) -> bool);
define_hook!(JavascriptParserStatement: SyncBail(parser: &mut JavascriptParser, stmt: Statement) -> bool);
define_hook!(JavascriptParserUnusedStatement: SyncBail(parser: &mut JavascriptParser, stmt: Statement) -> bool);
define_hook!(JavascriptParserModuleDeclaration: SyncBail(parser: &mut JavascriptParser, decl: &swc_core::ecma::ast::ModuleDecl) -> bool);
define_hook!(JavascriptParserBlockPreModuleDeclaration: SyncBail(parser: &mut JavascriptParser, decl: &swc_core::ecma::ast::ModuleDecl) -> bool);
define_hook!(JavascriptParserPreDeclarator: SyncBail(parser: &mut JavascriptParser, declarator: &VarDeclarator, declaration: VariableDeclaration<'_>) -> bool);
define_hook!(JavascriptParserEvaluateIdentifier: SyncBail(parser: &mut JavascriptParser, for_name: &str, start: u32, end: u32) -> BasicEvaluatedExpression<'static>);
define_hook!(JavascriptParserCanCollectDestructuringAssignmentProperties: SyncBail(parser: &mut JavascriptParser, expr: &Expr) -> bool);
define_hook!(JavascriptParserPattern: SyncBail(parser: &mut JavascriptParser, ident: &swc_core::ecma::ast::Ident, for_name: &str) -> bool);
define_hook!(JavascriptParserCall: SyncBail(parser: &mut JavascriptParser, expr: &CallExpr, for_name: &str) -> bool);
define_hook!(JavascriptParserCallMemberChain: SyncBail(parser: &mut JavascriptParser, expr: &CallExpr, for_name: &str, members: &[Atom], members_optionals: &[bool], member_ranges: &[Span]) -> bool);
define_hook!(JavascriptParserMember: SyncBail(parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::MemberExpr, for_name: &str) -> bool);
define_hook!(JavascriptParserMemberChain: SyncBail(parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::MemberExpr, for_name: &str, members: &[Atom], members_optionals: &[bool], member_ranges: &[Span]) -> bool);
define_hook!(JavascriptParserUnhandledExpressionMemberChain: SyncBail(parser: &mut JavascriptParser, root_info: &ExportedVariableInfo, expr: &MemberExpr) -> bool);
define_hook!(JavascriptParserMemberChainOfCallMemberChain: SyncBail(parser: &mut JavascriptParser, member_expr: &MemberExpr, callee_members: &[Atom], call_expr: &CallExpr, members: &[Atom], member_ranges: &[Span], for_name: &str) -> bool);
define_hook!(JavascriptParserCallMemberChainOfCallMemberChain: SyncBail(parser: &mut JavascriptParser, call_expr: &CallExpr, callee_members: &[Atom], inner_call_expr: &CallExpr, members: &[Atom], member_ranges: &[Span], for_name: &str) -> bool);
define_hook!(JavascriptParserTypeof: SyncBail(parser: &mut JavascriptParser, expr: &UnaryExpr, for_name: &str) -> bool);
define_hook!(JavascriptParserExpressionLogicalOperator: SyncBail(parser: &mut JavascriptParser, expr: &BinExpr) -> bool);
define_hook!(JavascriptParserBinaryExpression: SyncBail(parser: &mut JavascriptParser, expr: &BinExpr) -> bool);
define_hook!(JavascriptParserStatementIf: SyncBail(parser: &mut JavascriptParser, expr: &IfStmt) -> bool);
define_hook!(JavascriptParserClassExtendsExpression: SyncBail(parser: &mut JavascriptParser, super_class: &Expr, class_decl_or_expr: ClassDeclOrExpr) -> bool);
define_hook!(JavascriptParserClassBodyElement: SyncBail(parser: &mut JavascriptParser, element: &ClassMember, class_decl_or_expr: ClassDeclOrExpr) -> bool);
define_hook!(JavascriptParserClassBodyValue: SyncBail(parser: &mut JavascriptParser, element: &ClassMember, expr_span: Span, class_decl_or_expr: ClassDeclOrExpr) -> bool);
define_hook!(JavascriptParserDeclarator: SyncBail(parser: &mut JavascriptParser, expr: &VarDeclarator, stmt: VariableDeclaration<'_>) -> bool);
define_hook!(JavascriptParserNewExpression: SyncBail(parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::NewExpr, for_name: &str) -> bool);
define_hook!(JavascriptParserIdentifier: SyncBail(parser: &mut JavascriptParser, ident: &swc_core::ecma::ast::Ident, for_name: &str) -> bool);
define_hook!(JavascriptParserThis: SyncBail(parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::ThisExpr, for_name: &str) -> bool);
define_hook!(JavascriptParserAssign: SyncBail(parser: &mut JavascriptParser, expr: &AssignExpr, for_name: &str) -> bool);
define_hook!(JavascriptParserAssignMemberChain: SyncBail(parser: &mut JavascriptParser, expr: &AssignExpr, members: &[Atom], for_name: &str) -> bool);
define_hook!(JavascriptParserImportCall: SyncBail(parser: &mut JavascriptParser, expr: &CallExpr, import_then: Option<&CallExpr>, members: Option<(&[Atom], bool)>) -> bool);
define_hook!(JavascriptParserMetaProperty: SyncBail(parser: &mut JavascriptParser, root_name: &Atom, span: Span) -> bool);
define_hook!(JavascriptParserImport: SyncBail(parser: &mut JavascriptParser, statement: &swc_core::ecma::ast::ImportDecl, source: &str) -> bool);
define_hook!(JavascriptParserImportSpecifier: SyncBail(parser: &mut JavascriptParser, statement: &swc_core::ecma::ast::ImportDecl, source: &swc_core::atoms::Atom, export_name: Option<&Atom>, identifier_name: &Atom) -> bool);
define_hook!(JavascriptParserExportImport: SyncBail(parser: &mut JavascriptParser, statement: ExportImport, source: &Atom) -> bool);
define_hook!(JavascriptParserExport: SyncBail(parser: &mut JavascriptParser, statement: ExportLocal) -> bool);
define_hook!(JavascriptParserExportImportSpecifier: SyncBail(parser: &mut JavascriptParser, statement: ExportImport, source: &Atom, local_id: Option<&Atom>, export_name: Option<&Atom>, export_name_span: Option<Span>) -> bool);
define_hook!(JavascriptParserExportSpecifier: SyncBail(parser: &mut JavascriptParser, statement: ExportLocal, local_id: &Atom, export_name: &Atom, export_name_span: Span) -> bool);
define_hook!(JavascriptParserExportExpression: SyncBail(parser: &mut JavascriptParser, statement: ExportDefaultDeclaration, expr: ExportDefaultExpression) -> bool);
define_hook!(JavascriptParserOptionalChaining: SyncBail(parser: &mut JavascriptParser, expr: &OptChainExpr) -> bool);
define_hook!(JavascriptParserExpressionConditionalOperation: SyncBail(parser: &mut JavascriptParser, expr: &CondExpr) -> bool);
define_hook!(JavascriptParserFinish: SyncBail(parser: &mut JavascriptParser) -> bool);
define_hook!(JavascriptParserIsPure: SyncBail(parser: &mut JavascriptParser, expr: &Expr) -> bool);
define_hook!(JavascriptParserImportMetaPropertyInDestructuring: SyncBail(parser: &mut JavascriptParser, property: &DestructuringAssignmentProperty) -> String);

macro_rules! define_manual_sync_bail_hook {
  ($trait_name:ident, $hook_name:ident, ($($arg:ident : $arg_ty:ty),* $(,)?) -> $ret:ty) => {
    pub trait $trait_name {
      fn run<'a>(&self, $($arg: $arg_ty),*) -> rspack_error::Result<Option<$ret>>;

      fn stage(&self) -> i32 {
        0
      }
    }

    pub struct $hook_name {
      taps: Vec<Box<dyn $trait_name + Send + Sync>>,
      interceptors: Vec<Box<dyn Interceptor<Self> + Send + Sync>>,
    }

    impl Hook for $hook_name {
      type Tap = Box<dyn $trait_name + Send + Sync>;

      fn used_stages(&self) -> rspack_hook::__macro_helper::FxHashSet<i32> {
        rspack_hook::__macro_helper::FxHashSet::from_iter(self.taps.iter().map(|h| h.stage()))
      }

      fn intercept(
        &mut self,
        interceptor: impl Interceptor<Self> + Send + Sync + 'static,
      ) {
        self.interceptors.push(Box::new(interceptor));
      }
    }

    impl std::fmt::Debug for $hook_name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, stringify!($hook_name))
      }
    }

    impl Default for $hook_name {
      fn default() -> Self {
        Self {
          taps: Default::default(),
          interceptors: Default::default(),
        }
      }
    }

    impl $hook_name {
      pub fn call<'a>(&self, $($arg: $arg_ty),*) -> rspack_error::Result<Option<$ret>> {
        let mut additional_taps = std::vec::Vec::new();
        for interceptor in self.interceptors.iter() {
          additional_taps.extend(interceptor.call_blocking(self)?);
        }
        let mut all_taps = std::vec::Vec::with_capacity(self.taps.len() + additional_taps.len());
        all_taps.extend(self.taps.iter());
        all_taps.extend(additional_taps.iter());
        all_taps.sort_by_key(|hook| hook.stage());

        for tap in all_taps {
          if let Some(res) = tap.run($($arg),*)? {
            return Ok(Some(res));
          }
        }

        Ok(None)
      }

      pub fn tap(&mut self, tap: impl $trait_name + Send + Sync + 'static) {
        self.taps.push(Box::new(tap));
      }
    }
  };
}

define_manual_sync_bail_hook!(
  JavascriptParserEvaluate,
  JavascriptParserEvaluateHook,
  (parser: &mut JavascriptParser, expr: &'a Expr) -> BasicEvaluatedExpression<'a>
);
define_manual_sync_bail_hook!(
  JavascriptParserEvaluateTypeof,
  JavascriptParserEvaluateTypeofHook,
  (parser: &mut JavascriptParser, expr: &'a UnaryExpr, for_name: &str) -> BasicEvaluatedExpression<'a>
);
define_manual_sync_bail_hook!(
  JavascriptParserEvaluateTypeofAny,
  JavascriptParserEvaluateTypeofAnyHook,
  (parser: &mut JavascriptParser, expr: &'a UnaryExpr, for_name: &str) -> BasicEvaluatedExpression<'a>
);
define_manual_sync_bail_hook!(
  JavascriptParserEvaluateCallExpression,
  JavascriptParserEvaluateCallExpressionHook,
  (parser: &mut JavascriptParser, name: &str, expr: &'a CallExpr) -> BasicEvaluatedExpression<'a>
);

pub trait JavascriptParserEvaluateCallExpressionMember {
  fn run<'a>(
    &self,
    parser: &mut JavascriptParser,
    property: &str,
    expr: &'a CallExpr,
    param: BasicEvaluatedExpression<'a>,
  ) -> rspack_error::Result<Option<BasicEvaluatedExpression<'a>>>;

  fn stage(&self) -> i32 {
    0
  }
}

pub struct JavascriptParserEvaluateCallExpressionMemberHook {
  taps: Vec<Box<dyn JavascriptParserEvaluateCallExpressionMember + Send + Sync>>,
  interceptors: Vec<Box<dyn Interceptor<Self> + Send + Sync>>,
}

impl Hook for JavascriptParserEvaluateCallExpressionMemberHook {
  type Tap = Box<dyn JavascriptParserEvaluateCallExpressionMember + Send + Sync>;

  fn used_stages(&self) -> rspack_hook::__macro_helper::FxHashSet<i32> {
    rspack_hook::__macro_helper::FxHashSet::from_iter(self.taps.iter().map(|h| h.stage()))
  }

  fn intercept(&mut self, interceptor: impl Interceptor<Self> + Send + Sync + 'static) {
    self.interceptors.push(Box::new(interceptor));
  }
}

impl std::fmt::Debug for JavascriptParserEvaluateCallExpressionMemberHook {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "JavascriptParserEvaluateCallExpressionMemberHook")
  }
}

impl Default for JavascriptParserEvaluateCallExpressionMemberHook {
  fn default() -> Self {
    Self {
      taps: Default::default(),
      interceptors: Default::default(),
    }
  }
}

impl JavascriptParserEvaluateCallExpressionMemberHook {
  pub fn call<'a>(
    &self,
    parser: &mut JavascriptParser,
    property: &str,
    expr: &'a CallExpr,
    param: BasicEvaluatedExpression<'a>,
  ) -> rspack_error::Result<Option<BasicEvaluatedExpression<'a>>> {
    let mut additional_taps = std::vec::Vec::new();
    for interceptor in self.interceptors.iter() {
      additional_taps.extend(interceptor.call_blocking(self)?);
    }
    let mut all_taps = std::vec::Vec::with_capacity(self.taps.len() + additional_taps.len());
    all_taps.extend(self.taps.iter());
    all_taps.extend(additional_taps.iter());
    all_taps.sort_by_key(|hook| hook.stage());

    for tap in all_taps {
      if let Some(res) = tap.run(parser, property, expr, param.clone())? {
        return Ok(Some(res));
      }
    }

    Ok(None)
  }

  pub fn tap(
    &mut self,
    tap: impl JavascriptParserEvaluateCallExpressionMember + Send + Sync + 'static,
  ) {
    self.taps.push(Box::new(tap));
  }
}

macro_rules! call_sync {
  ($field:ident($($arg:ident: $ty:ty),* $(,)?)) => {
    pub fn $field(&self, $($arg: $ty),*) {
      self
        .$field
        .call($($arg),*)
        .expect(concat!(stringify!($field), " parser hook should not fail"));
    }
  };
}

macro_rules! call_sync_bail {
  ($field:ident($($arg:ident: $ty:ty),* $(,)?) -> $ret:ty) => {
    pub fn $field(&self, $($arg: $ty),*) -> Option<$ret> {
      self
        .$field
        .call($($arg),*)
        .expect(concat!(stringify!($field), " parser hook should not fail"))
    }
  };
}

macro_rules! call_sync_bail_map {
  ($field:ident($($arg:ident: $ty:ty),* $(,)?), $for_name:ident -> $ret:ty) => {
    pub fn $field(&self, $($arg: $ty),*) -> Option<$ret> {
      self
        .$field
        .get($for_name)
        .and_then(|hook| {
          hook
            .call($($arg),*)
            .expect(concat!(stringify!($field), " parser hook should not fail"))
        })
    }
  };
}

#[derive(Debug, Default)]
pub struct JavascriptParserHooks {
  pub pre_statement: JavascriptParserPreStatementHook,
  pub block_pre_statement: JavascriptParserBlockPreStatementHook,
  pub top_level_await_expr: JavascriptParserTopLevelAwaitExprHook,
  pub top_level_for_of_await_stmt: JavascriptParserTopLevelForOfAwaitStmtHook,
  pub can_rename: HookMap<String, JavascriptParserCanRenameHook>,
  pub rename: HookMap<String, JavascriptParserRenameHook>,
  pub program: JavascriptParserProgramHook,
  pub statement: JavascriptParserStatementHook,
  pub unused_statement: JavascriptParserUnusedStatementHook,
  pub module_declaration: JavascriptParserModuleDeclarationHook,
  pub block_pre_module_declaration: JavascriptParserBlockPreModuleDeclarationHook,
  pub pre_declarator: JavascriptParserPreDeclaratorHook,
  pub evaluate: JavascriptParserEvaluateHook,
  pub evaluate_typeof: HookMap<String, JavascriptParserEvaluateTypeofHook>,
  pub evaluate_typeof_any: JavascriptParserEvaluateTypeofAnyHook,
  pub evaluate_call_expression: JavascriptParserEvaluateCallExpressionHook,
  pub evaluate_call_expression_member: JavascriptParserEvaluateCallExpressionMemberHook,
  pub evaluate_identifier: HookMap<String, JavascriptParserEvaluateIdentifierHook>,
  pub can_collect_destructuring_assignment_properties:
    JavascriptParserCanCollectDestructuringAssignmentPropertiesHook,
  pub pattern: HookMap<String, JavascriptParserPatternHook>,
  pub call: HookMap<String, JavascriptParserCallHook>,
  pub call_member_chain: HookMap<String, JavascriptParserCallMemberChainHook>,
  pub member: HookMap<String, JavascriptParserMemberHook>,
  pub member_chain: HookMap<String, JavascriptParserMemberChainHook>,
  pub unhandled_expression_member_chain: JavascriptParserUnhandledExpressionMemberChainHook,
  pub member_chain_of_call_member_chain:
    HookMap<String, JavascriptParserMemberChainOfCallMemberChainHook>,
  pub call_member_chain_of_call_member_chain:
    HookMap<String, JavascriptParserCallMemberChainOfCallMemberChainHook>,
  pub r#typeof: HookMap<String, JavascriptParserTypeofHook>,
  pub expression_logical_operator: JavascriptParserExpressionLogicalOperatorHook,
  pub binary_expression: JavascriptParserBinaryExpressionHook,
  pub statement_if: JavascriptParserStatementIfHook,
  pub class_extends_expression: JavascriptParserClassExtendsExpressionHook,
  pub class_body_element: JavascriptParserClassBodyElementHook,
  pub class_body_value: JavascriptParserClassBodyValueHook,
  pub declarator: JavascriptParserDeclaratorHook,
  pub new_expression: HookMap<String, JavascriptParserNewExpressionHook>,
  pub identifier: HookMap<String, JavascriptParserIdentifierHook>,
  pub this: HookMap<String, JavascriptParserThisHook>,
  pub assign: HookMap<String, JavascriptParserAssignHook>,
  pub assign_member_chain: HookMap<String, JavascriptParserAssignMemberChainHook>,
  pub import_call: JavascriptParserImportCallHook,
  pub meta_property: JavascriptParserMetaPropertyHook,
  pub import: JavascriptParserImportHook,
  pub import_specifier: JavascriptParserImportSpecifierHook,
  pub export_import: JavascriptParserExportImportHook,
  pub export: JavascriptParserExportHook,
  pub export_import_specifier: JavascriptParserExportImportSpecifierHook,
  pub export_specifier: JavascriptParserExportSpecifierHook,
  pub export_expression: JavascriptParserExportExpressionHook,
  pub optional_chaining: JavascriptParserOptionalChainingHook,
  pub expression_conditional_operation: JavascriptParserExpressionConditionalOperationHook,
  pub finish: JavascriptParserFinishHook,
  pub is_pure: JavascriptParserIsPureHook,
  pub import_meta_property_in_destructuring: JavascriptParserImportMetaPropertyInDestructuringHook,
}

impl JavascriptParserHooks {
  pub fn new(
    plugins: Vec<BoxJavascriptParserPlugin>,
    compiler_options: &CompilerOptions,
    javascript_options: &JavascriptParserOptions,
    parser_runtime_requirements: &ParserRuntimeRequirementsData,
    module_type: &ModuleType,
    module_layer: Option<&ModuleLayer>,
    resource_data: &ResourceData,
  ) -> Self {
    let mut hooks = Self::default();
    let mut context = JavascriptParserPluginContext {
      hooks: &mut hooks,
      compiler_options,
      javascript_options,
      parser_runtime_requirements,
      module_type,
      module_layer,
      resource_data,
    };
    for plugin in plugins {
      plugin.apply(&mut context);
    }
    hooks
  }

  call_sync!(top_level_await_expr(parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::AwaitExpr));
  call_sync!(top_level_for_of_await_stmt(parser: &mut JavascriptParser, stmt: &swc_core::ecma::ast::ForOfStmt));

  call_sync_bail!(program(parser: &mut JavascriptParser, ast: &swc_core::ecma::ast::Program) -> bool);
  call_sync_bail!(finish(parser: &mut JavascriptParser) -> bool);
  call_sync_bail!(block_pre_module_declaration(parser: &mut JavascriptParser, decl: &swc_core::ecma::ast::ModuleDecl) -> bool);
  call_sync_bail!(module_declaration(parser: &mut JavascriptParser, decl: &swc_core::ecma::ast::ModuleDecl) -> bool);
  call_sync_bail!(can_collect_destructuring_assignment_properties(parser: &mut JavascriptParser, expr: &Expr) -> bool);
  call_sync_bail!(unhandled_expression_member_chain(parser: &mut JavascriptParser, root_info: &ExportedVariableInfo, expr: &MemberExpr) -> bool);
  call_sync_bail!(expression_logical_operator(parser: &mut JavascriptParser, expr: &BinExpr) -> bool);
  call_sync_bail!(binary_expression(parser: &mut JavascriptParser, expr: &BinExpr) -> bool);
  call_sync_bail!(statement(parser: &mut JavascriptParser, stmt: Statement) -> bool);
  call_sync_bail!(unused_statement(parser: &mut JavascriptParser, stmt: Statement) -> bool);
  call_sync_bail!(statement_if(parser: &mut JavascriptParser, expr: &IfStmt) -> bool);
  call_sync_bail!(declarator(parser: &mut JavascriptParser, expr: &VarDeclarator, stmt: VariableDeclaration<'_>) -> bool);
  call_sync_bail!(class_extends_expression(parser: &mut JavascriptParser, super_class: &Expr, class_decl_or_expr: ClassDeclOrExpr) -> bool);
  call_sync_bail!(class_body_element(parser: &mut JavascriptParser, element: &ClassMember, class_decl_or_expr: ClassDeclOrExpr) -> bool);
  call_sync_bail!(class_body_value(parser: &mut JavascriptParser, element: &ClassMember, expr_span: Span, class_decl_or_expr: ClassDeclOrExpr) -> bool);
  call_sync_bail!(pre_declarator(parser: &mut JavascriptParser, declarator: &VarDeclarator, declaration: VariableDeclaration<'_>) -> bool);
  call_sync_bail!(pre_statement(parser: &mut JavascriptParser, stmt: Statement) -> bool);
  call_sync_bail!(block_pre_statement(parser: &mut JavascriptParser, stmt: Statement) -> bool);
  call_sync_bail!(import_call(parser: &mut JavascriptParser, expr: &CallExpr, import_then: Option<&CallExpr>, members: Option<(&[Atom], bool)>) -> bool);
  call_sync_bail!(meta_property(parser: &mut JavascriptParser, root_name: &Atom, span: Span) -> bool);
  call_sync_bail!(import(parser: &mut JavascriptParser, statement: &swc_core::ecma::ast::ImportDecl, source: &str) -> bool);
  call_sync_bail!(import_specifier(parser: &mut JavascriptParser, statement: &swc_core::ecma::ast::ImportDecl, source: &swc_core::atoms::Atom, export_name: Option<&Atom>, identifier_name: &Atom) -> bool);
  call_sync_bail!(export_import(parser: &mut JavascriptParser, statement: ExportImport, source: &Atom) -> bool);
  call_sync_bail!(export(parser: &mut JavascriptParser, statement: ExportLocal) -> bool);
  call_sync_bail!(export_import_specifier(parser: &mut JavascriptParser, statement: ExportImport, source: &Atom, local_id: Option<&Atom>, export_name: Option<&Atom>, export_name_span: Option<Span>) -> bool);
  call_sync_bail!(export_specifier(parser: &mut JavascriptParser, statement: ExportLocal, local_id: &Atom, export_name: &Atom, export_name_span: Span) -> bool);
  call_sync_bail!(export_expression(parser: &mut JavascriptParser, statement: ExportDefaultDeclaration, expr: ExportDefaultExpression) -> bool);
  call_sync_bail!(optional_chaining(parser: &mut JavascriptParser, expr: &OptChainExpr) -> bool);
  call_sync_bail!(expression_conditional_operation(parser: &mut JavascriptParser, expr: &CondExpr) -> bool);
  call_sync_bail!(is_pure(parser: &mut JavascriptParser, expr: &Expr) -> bool);
  call_sync_bail!(import_meta_property_in_destructuring(parser: &mut JavascriptParser, property: &DestructuringAssignmentProperty) -> String);

  call_sync_bail_map!(member(parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::MemberExpr, for_name: &str), for_name -> bool);
  call_sync_bail_map!(member_chain(parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::MemberExpr, for_name: &str, members: &[Atom], members_optionals: &[bool], member_ranges: &[Span]), for_name -> bool);
  call_sync_bail_map!(call_member_chain(parser: &mut JavascriptParser, expr: &CallExpr, for_name: &str, members: &[Atom], members_optionals: &[bool], member_ranges: &[Span]), for_name -> bool);
  call_sync_bail_map!(member_chain_of_call_member_chain(parser: &mut JavascriptParser, member_expr: &MemberExpr, callee_members: &[Atom], call_expr: &CallExpr, members: &[Atom], member_ranges: &[Span], for_name: &str), for_name -> bool);
  call_sync_bail_map!(call_member_chain_of_call_member_chain(parser: &mut JavascriptParser, call_expr: &CallExpr, callee_members: &[Atom], inner_call_expr: &CallExpr, members: &[Atom], member_ranges: &[Span], for_name: &str), for_name -> bool);
  call_sync_bail_map!(assign(parser: &mut JavascriptParser, expr: &AssignExpr, for_name: &str), for_name -> bool);
  call_sync_bail_map!(assign_member_chain(parser: &mut JavascriptParser, expr: &AssignExpr, members: &[Atom], for_name: &str), for_name -> bool);
  call_sync_bail_map!(r#typeof(parser: &mut JavascriptParser, expr: &UnaryExpr, for_name: &str), for_name -> bool);
  call_sync_bail_map!(new_expression(parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::NewExpr, for_name: &str), for_name -> bool);
  call_sync_bail_map!(identifier(parser: &mut JavascriptParser, ident: &swc_core::ecma::ast::Ident, for_name: &str), for_name -> bool);
  call_sync_bail_map!(this(parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::ThisExpr, for_name: &str), for_name -> bool);
  call_sync_bail_map!(evaluate_identifier(parser: &mut JavascriptParser, for_name: &str, start: u32, end: u32), for_name -> BasicEvaluatedExpression<'static>);
  call_sync_bail_map!(pattern(parser: &mut JavascriptParser, ident: &swc_core::ecma::ast::Ident, for_name: &str), for_name -> bool);
  call_sync_bail_map!(call(parser: &mut JavascriptParser, expr: &CallExpr, for_name: &str), for_name -> bool);
  call_sync_bail_map!(can_rename(parser: &mut JavascriptParser, for_name: &str), for_name -> bool);
  call_sync_bail_map!(rename(parser: &mut JavascriptParser, expr: &Expr, for_name: &str), for_name -> bool);

  pub fn evaluate<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a Expr,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    self
      .evaluate
      .call(parser, expr)
      .expect("evaluate parser hook should not fail")
  }

  pub fn evaluate_call_expression<'a>(
    &self,
    parser: &mut JavascriptParser,
    name: &str,
    expr: &'a CallExpr,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    self
      .evaluate_call_expression
      .call(parser, name, expr)
      .expect("evaluate_call_expression parser hook should not fail")
  }

  pub fn evaluate_call_expression_member<'a>(
    &self,
    parser: &mut JavascriptParser,
    property: &str,
    expr: &'a CallExpr,
    param: BasicEvaluatedExpression<'a>,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    self
      .evaluate_call_expression_member
      .call(parser, property, expr, param)
      .expect("evaluate_call_expression_member parser hook should not fail")
  }

  pub fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    self
      .evaluate_typeof
      .get(for_name)
      .and_then(|hook| {
        hook
          .call(parser, expr, for_name)
          .expect("evaluate_typeof parser hook should not fail")
      })
      .or_else(|| {
        self
          .evaluate_typeof_any
          .call(parser, expr, for_name)
          .expect("evaluate_typeof_any parser hook should not fail")
      })
  }
}

pub type JavaScriptParserPluginDrive = JavascriptParserHooks;

impl JavascriptParserHooks {
  pub fn call_member_chain_from_call_expr(
    &self,
    parser: &mut JavascriptParser,
    expr: &CallExpr,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    member_ranges: &[Span],
  ) -> Option<bool> {
    assert!(matches!(expr.callee, Callee::Expr(_)));
    self.call_member_chain(
      parser,
      expr,
      for_name,
      members,
      members_optionals,
      member_ranges,
    )
  }
}
