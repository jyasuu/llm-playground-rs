use yew::prelude::*;

mod llm_playground;
use llm_playground::FlexibleLLMPlayground;

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
