use std::sync::Arc;

use rspack_core::OverrideStrict;
use swc_core::ecma::ast::Program;

use super::{JavascriptParserPlugin, JavascriptParserPluginContext, JavascriptParserProgram};
use crate::visitors::JavascriptParser;

#[derive(Default)]
pub struct OverrideStrictPlugin;

impl OverrideStrictPlugin {
  fn program(&self, parser: &mut JavascriptParser, _: &Program) -> Option<bool> {
    if let Some(strict) = parser.javascript_options.override_strict {
      parser.build_info.strict = matches!(strict, OverrideStrict::Strict);
    }

    None
  }
}

crate::impl_javascript_parser_hook!(
  OverrideStrictPlugin,
  JavascriptParserProgram,
  program(parser: &mut JavascriptParser, ast: &Program) -> bool
);

impl JavascriptParserPlugin for OverrideStrictPlugin {
  fn apply(self: Arc<Self>, context: &mut JavascriptParserPluginContext<'_>) {
    context.hooks.program.tap(self);
  }
}
