use yew::prelude::*;

mod llm_playground;
use llm_playground::FlexibleLLMPlaygroundSimple;

#[function_component(App)]
fn app() -> Html {
    html! {
        // <LLMPlayground />
        <FlexibleLLMPlaygroundSimple />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
