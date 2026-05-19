use leptos::prelude::*;
use leptos::svg;

#[component]
pub fn Icon(icon: icondata::Icon, #[prop(optional, into)] class: String) -> impl IntoView {
    let data = format!("<g>{}</g>", icon.data);
    svg::svg()
        .attr("class", class)
        .attr("viewBox", icon.view_box)
        .attr("stroke-linecap", icon.stroke_linecap)
        .attr("stroke-linejoin", icon.stroke_linejoin)
        .attr("stroke-width", icon.stroke_width)
        .attr("stroke", icon.stroke)
        .attr("fill", icon.fill.unwrap_or("currentColor"))
        .attr("role", "graphics-symbol")
        .child(svg::InertElement::new(data))
}
