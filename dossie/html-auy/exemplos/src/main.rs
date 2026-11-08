//! Exemplo mínimo e executável do `html-auy`.
//!
//! O crate **não é editor**: ele não tem documento editável, seleção, undo nem DOM. Ele recebe
//! dados já decididos e devolve bytes de HTML determinísticos. O que este exemplo demonstra:
//!
//! 1. `html!` valida elemento/atributo/aninhamento em tempo de compilação;
//! 2. o escape é escolhido pelo *tipo* do valor (`Text`, `Alt`, `Href`…), não no ponto de escrita;
//! 3. primitivos (`Img`, `Field`) exigem o dado de acessibilidade como parâmetro;
//! 4. `Page` é a única porta para `html/head/body/main`;
//! 5. mesmos dados ⇒ mesmos bytes (base de um `ETag` estável).
//!
//! Rodar: `cargo run --offline` (a partir desta pasta).

use html_auy::{
    Alt, AttrVal, Class, Field, FieldName, FieldState, HeightPx, Href, Img, Lang, LUrl, Markup,
    MediaPath, Msg, Page, ShellData, Src, Text, View, WidthPx, html,
};

/// Catálogo de mensagens do consumidor. O crate só conhece o trait `Msg` — locale é do consumidor.
#[derive(Clone, Copy)]
enum Etiqueta {
    Nome,
}

impl Msg for Etiqueta {
    fn as_str(self) -> &'static str {
        match self {
            Self::Nome => "Nome",
        }
    }
}

/// Storage de mídia do consumidor: entrega caminho já validado, nunca `String` cru.
struct Midia(LUrl<'static>);

impl MediaPath for Midia {
    fn path(&self) -> &LUrl<'_> {
        &self.0
    }
}

/// A view inteira: chamada duas vezes para provar que a saída é função só dos dados.
fn fragmento(midia: &Midia) -> Markup {
    html! {
        section class=(Class::new("card flat")) {
            h2 { (Text::new("Vendas <trimestre> & cia")) }
            p {
                (Text::new("aspas \" e apóstrofo ' — o texto é escapado, não interpretado"))
            }
            a href=(Href::internal(LUrl::new("/relatorios/2026").expect("rota fixa"))) {
                (Text::new("Ver relatório"))
            }
            img src=(Src::media(midia))
                alt=(Alt::new("Gráfico de vendas"))
                width=(WidthPx::new(720))
                height=(HeightPx::new(480));
        }
    }
}

fn main() {
    let midia = Midia(LUrl::from_string(String::from("/media/grafico.avif")).expect("rota fixa"));

    println!("== 1. fragmento (`html!`) ==\n{}\n", fragmento(&midia).as_str());

    // `Img::new` não tem default: `alt` e as duas dimensões são parâmetros obrigatórios.
    let figura = Img::new(
        Src::media(&midia),
        Alt::decorative(),
        WidthPx::new(64),
        HeightPx::new(64),
    )
    .render();
    println!("== 2. primitivo (`Img`) ==\n{}\n", figura.as_str());

    // `Field` deriva `label`, `for`, `id`, `aria-describedby` e `aria-invalid` do estado.
    let estado = FieldState::new(AttrVal::new("Ana"))
        .required(true)
        .help(Text::new("Como você quer ser chamado?"));
    println!(
        "== 3. campo (`Field::text`) ==\n{}\n",
        Field::text(FieldName::new("nome"), Etiqueta::Nome, &estado).as_str()
    );

    // `Page` emite DOCTYPE, `<html lang>`, `<head>`, região viva e `<main id="main">`.
    let dados = ShellData::new(
        Text::new("Demo html-auy"),
        Text::new("Demonstração"),
        Lang::parse("pt-BR").expect("idioma"),
    );
    let primeira = Page::new(&dados, View::new(fragmento(&midia)));
    let segunda = Page::new(&dados, View::new(fragmento(&midia)));

    assert_eq!(primeira.markup().as_str(), segunda.markup().as_str());
    println!("== 4. página (`Page`) ==\n{}\n", primeira.markup().as_str());
    println!(
        "== 5. determinismo ==\nmesmos dados => mesmos bytes ({} bytes)",
        primeira.markup().len()
    );
}
