use std::collections::HashMap;
use crate::agent::Agent;
use anyhow::{anyhow, Result};
use tracing::info;

/// A pipeline of agents executed as a Directed Acyclic Graph (DAG)
pub struct AgentPipeline {
    agents: HashMap<String, Agent>,
    /// Adjacency list: "agent_name" -> vec!["next_agent_1", "next_agent_2"]
    /// Currently supports linear chains or simple branching logic managed by the runner.
    flow: HashMap<String, Vec<String>>,
    start_node: Option<String>,
}

pub struct PipelineBuilder {
    agents: HashMap<String, Agent>,
    flow: HashMap<String, Vec<String>>,
    start_node: Option<String>,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            flow: HashMap::new(),
            start_node: None,
        }
    }

    pub fn agent(mut self, id: impl Into<String>, agent: Agent) -> Self {
        self.agents.insert(id.into(), agent);
        self
    }

    /// Define the start node
    pub fn start(mut self, id: impl Into<String>) -> Self {
        self.start_node = Some(id.into());
        self
    }

    /// Define a transition: from -> to
    pub fn transition(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        let from = from.into();
        let to = to.into();
        self.flow.entry(from).or_default().push(to);
        self
    }

    pub fn build(self) -> Result<AgentPipeline> {
        Ok(AgentPipeline {
            agents: self.agents,
            flow: self.flow,
            start_node: self.start_node,
        })
    }
}

impl AgentPipeline {
    pub fn builder() -> PipelineBuilder {
        PipelineBuilder::new()
    }

    /// Run the pipeline with an initial input.
    /// This is a simplified runner that supports linear chains: A -> B -> C.
    /// The output of A becomes the input of B.
    pub async fn run(&self, initial_input: &str) -> Result<String> {
        let mut current_node = self.start_node.clone()
            .ok_or_else(|| anyhow!("No start node defined in pipeline"))?;
        
        let mut current_input = initial_input.to_string();

        loop {
            let agent = self.agents.get(&current_node)
                .ok_or_else(|| anyhow!("Agent '{}' not found", current_node))?;

            info!("--- [Pipeline] Running Agent: {} ---", current_node);
            
            // Run the agent
            let output = agent.run(&current_input).await?;
            current_input = output.clone();

            // Find next node
            if let Some(next_nodes) = self.flow.get(&current_node) {
                if next_nodes.is_empty() {
                    break; // End of chain
                }
                // For linear pipeline, pick the first one.
                // TODO: Implement advanced routing/branching logic.
                current_node = next_nodes[0].clone();
            } else {
                break; // No outgoing transitions
            }
        }

        Ok(current_input)
    }
}
