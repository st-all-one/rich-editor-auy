//! Casos que o `html!` recusa em tempo de compilação. Este pacote existe para que a saída de erro
//! fique registrada no dossiê — ele nunca deve compilar.
//!
//! Rodar: `cargo check --offline 2>&1` (a partir desta pasta).

use html_auy::{Text, html};

fn main() {
    // 1. atributo proibido (event handler)
    let _ = html! { div onclick="go()" { } };

    // 2. atributo booleano com literal: `disabled="false"` seria VERDADEIRO em HTML
    let _ = html! { button type="button" disabled="false" { (Text::new("x")) } };

    // 3. atributo tipado não aceita literal — `alt` é `Alt`
    let _ = html! { span alt="x" { } };

    // 4. conteúdo-modelo: `div` (fluxo) dentro de `p` (só phrasing)
    let _ = html! { p { div { } } };

    // 5. elemento de documento: `<form>` só existe via `Form::open`
    let _ = html! { form { } };
}
