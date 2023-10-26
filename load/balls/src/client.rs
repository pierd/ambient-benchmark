use ambient_api::prelude::*;

#[main]
pub fn main() {
    // packages::this::entity();
    // eprintln!("{:?}", Controls.el().spawn_tree().root_entity());
}

#[element_component]
pub fn Controls(_hooks: &mut Hooks) -> Element {
    Text::el("TODO Balls")
}
