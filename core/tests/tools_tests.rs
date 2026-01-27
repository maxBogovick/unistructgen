use unistructgen_core::{AiTool, ToolRegistry, ToolError, ToolResult, Context};
use serde_json::json;
use serde::Deserialize;
use unistructgen_core::async_trait;

// Manual implementation of a tool (this is what the macro will generate)
struct CalculatorTool;

#[derive(Deserialize)]
struct CalculatorArgs {
    a: i32,
    b: i32,
    op: String,
}

#[async_trait]
impl AiTool for CalculatorTool {
    fn name(&self) -> &str {
        "calculate"
    }

    fn description(&self) -> &str {
        "Performs basic arithmetic"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "a": { "type": "integer" },
                "b": { "type": "integer" },
                "op": { "type": "string", "enum": ["add", "sub"] }
            },
            "required": ["a", "b", "op"]
        })
    }

    async fn call(&self, arguments_json: &str, _context: &Context) -> ToolResult {
        let args: CalculatorArgs = serde_json::from_str(arguments_json)?;
        match args.op.as_str() {
            "add" => Ok((args.a + args.b).to_string()),
            "sub" => Ok((args.a - args.b).to_string()),
            _ => Err(ToolError::ExecutionError("Unknown operation".to_string())),
        }
    }
}

#[tokio::test]
async fn test_tool_registry() {
    let mut registry = ToolRegistry::new();
    registry.register(CalculatorTool);
    let context = Context::new();

    // 1. Check definitions
    let defs = registry.get_definitions();
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0]["function"]["name"], "calculate");

    // 2. Execute successfully
    let result = registry.execute("calculate", r#"{"a": 5, "b": 3, "op": "add"}"#, &context).await;
    assert_eq!(result.unwrap(), "8");

    // 3. Execute unknown tool
    let result = registry.execute("unknown", "{}", &context).await;
    assert!(matches!(result, Err(ToolError::NotFound(_))));

    // 4. Execute with bad args
    let result = registry.execute("calculate", r#"{"a": "bad"}"#, &context).await;
    assert!(matches!(result, Err(ToolError::ArgumentError(_))));
}
