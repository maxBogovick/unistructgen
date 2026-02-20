use unistructgen::agent::{Agent, AgentPipeline};
use unistructgen::core::ToolRegistry;
use unistructgen_macro::ai_tool;
use unistructgen::llm::{LlmClient, LlmClientFactory, Provider};
use std::sync::Arc;

// --- Tools ---

#[ai_tool]
fn get_weather(city: String) -> String {
    format!("Weather in {} is sunny, 25C", city)
}

#[ai_tool]
fn suggest_activities(weather: String) -> String {
    if weather.contains("sunny") {
        "Go for a walk, play tennis".to_string()
    } else {
        "Read a book, code Rust".to_string()
    }
}

// --- Demo ---

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // 1. Setup LLM (Auto-detect: OpenAI or Ollama)
    // For this demo to work without keys, assume Ollama is running or mock it.
    // If you have OPENAI_API_KEY env, it will use OpenAI.
    let client = LlmClientFactory::new()
        .with_provider(Provider::Auto)
        .with_model("gpt-4o") // or "llama3"
        .build()?;
        
    let client: Arc<dyn LlmClient> = Arc::from(client);

    // 2. Setup Registries
    let mut weather_registry = ToolRegistry::new();
    weather_registry.register(GetWeatherTool);

    let mut activity_registry = ToolRegistry::new();
    activity_registry.register(SuggestActivitiesTool);

    // 3. Create Agents
    
    // Agent 1: Researcher (has weather tool)
    let researcher = Agent::builder()
        .name("Researcher")
        .client(client.clone())
        .tools(Arc::new(weather_registry))
        .system_prompt("You are a researcher. Use the tool to find information.")
        .build()?;

    // Agent 2: Advisor (has activity tool)
    let advisor = Agent::builder()
        .name("Advisor")
        .client(client.clone())
        .tools(Arc::new(activity_registry))
        .system_prompt("You are a lifestyle advisor. Based on the input, suggest activities using your tool.")
        .build()?;

    // 4. Create Pipeline: Researcher -> Advisor
    let pipeline = AgentPipeline::builder()
        .agent("researcher", researcher)
        .agent("advisor", advisor)
        .start("researcher")
        .transition("researcher", "advisor")
        .build()?;

    println!("Starting pipeline...");
    
    // 5. Run
    // Input: "What should I do in New York today?"
    // Flow:
    // Researcher: Calls get_weather("New York") -> "Sunny 25C" -> Output: "The weather is sunny."
    // Advisor: Input "The weather is sunny." -> Calls suggest_activities -> "Go for a walk" -> Output.
    
    let result = pipeline.run("What should I do in New York today? Check the weather first.").await;
    
    match result {
        Ok(ans) => println!("\nFinal Recommendation:\n{}", ans),
        Err(e) => eprintln!("Pipeline failed: {}", e),
    }

    Ok(())
}
