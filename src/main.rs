use yew::prelude::*;

mod llm_playground;
// Original tightly coupled implementation:
// use llm_playground::FlexibleLLMPlayground;

// New decoupled event-driven implementation (has build issues, reverting temporarily):
use llm_playground::FlexibleLLMPlaygroundRefactored as FlexibleLLMPlayground;

#[function_component(App)]
fn app() -> Html {
    html! {
        // <LLMPlayground />
        <FlexibleLLMPlayground />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
