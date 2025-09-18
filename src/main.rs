use yew::prelude::*;

mod llm_playground;
use llm_playground::LLMPlayground;

#[function_component(App)]
fn app() -> Html {
    html! {
        <LLMPlayground />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
