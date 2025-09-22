use yew::prelude::*;

mod llm_playground;
use llm_playground::FlexibleLLMPlayground;

#[function_component(App)]
fn app() -> Html {
    html! {
        <FlexibleLLMPlayground />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}