//! Contrato de expansão do macro. **Não é API**: o nome muda sem aviso.
//!
//! `html!` compila para uma sequência de chamadas a [`Writer`]; cada método escreve um fragmento
//! já na forma canônica (ordem, escape, presença). Se este arquivo mudar, o macro muda junto.

use crate::escape;
use crate::markup::{Markup, Render};
use crate::msg::Msg;
use crate::value::{Alt, AttrValue, FieldName, Href, Id, Label, Sizes, Src, SrcSet, Timestamp};
use alloc::string::String;

/// Buffer de saída com as operações que o macro emite.
#[derive(Debug, Default)]
pub struct Writer {
    buf: String,
}

impl Writer {
    /// Buffer com capacidade reservada.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: String::with_capacity(capacity),
        }
    }

    /// Abre a tag (`<section`): atributos vêm em seguida.
    pub fn open(&mut self, tag: &'static str) {
        self.buf.push('<');
        self.buf.push_str(tag);
    }

    /// Fecha a tag de abertura (`>`).
    pub fn close_open(&mut self) {
        self.buf.push('>');
    }

    /// Fecha a tag de abertura em conteúdo estrangeiro (`<circle … />`): no HTML, `/>` só é honrado
    /// em SVG/MathML, e sem ele um segundo `circle` vira filho do primeiro.
    pub fn close_open_self(&mut self) {
        self.buf.push_str("/>");
    }

    /// Fecha o elemento (`</section>`).
    pub fn close(&mut self, tag: &'static str) {
        self.buf.push_str("</");
        self.buf.push_str(tag);
        self.buf.push('>');
    }

    /// Entrega o `Markup` final.
    #[must_use]
    pub fn finish(self) -> Markup {
        Markup(self.buf)
    }

    /// Atributo com valor escapado.
    pub fn attr(&mut self, name: &'static str, value: &impl AttrValue) {
        self.buf.push(' ');
        self.buf.push_str(name);
        self.buf.push_str("=\"");
        value.write_attr(&mut self.buf);
        self.buf.push('"');
    }

    /// Atributo de presença (`disabled`).
    pub fn attr_flag(&mut self, name: &'static str, on: bool) {
        if on {
            self.buf.push(' ');
            self.buf.push_str(name);
        }
    }

    /// Atributo de valor literal: vazio vira presença (`hidden`).
    pub fn attr_static(&mut self, name: &'static str, value: &'static str) {
        if value.is_empty() {
            self.attr_flag(name, true);
            return;
        }
        self.buf.push(' ');
        self.buf.push_str(name);
        self.buf.push_str("=\"");
        escape::attr(&mut self.buf, value);
        self.buf.push('"');
    }

    /// `class`, omitido quando o conjunto está vazio.
    pub fn attr_class(&mut self, value: &impl AttrValue) {
        if !value.is_empty_attr() {
            self.attr("class", value);
        }
    }

    /// `sizes`, omitido quando a lista é vazia (gramática inválida).
    pub fn attr_sizes(&mut self, value: &Sizes<'_>) {
        if !value.is_empty() {
            self.attr("sizes", value);
        }
    }

    /// `srcset`, omitido quando não há variante.
    pub fn attr_srcset(&mut self, value: &SrcSet<'_>) {
        if !value.is_empty() {
            self.attr("srcset", value);
        }
    }

    /// `href` de link.
    pub fn attr_href(&mut self, value: &Href<'_>) {
        self.attr("href", value);
    }

    /// `src` de mídia.
    pub fn attr_src(&mut self, value: &Src<'_>) {
        self.attr("src", value);
    }

    /// `alt` de imagem (vazio é decorativo e continua emitido).
    pub fn attr_alt(&mut self, value: &Alt<'_>) {
        self.attr("alt", value);
    }

    /// `aria-label`.
    pub fn attr_label(&mut self, value: &Label<'_>) {
        self.attr("aria-label", value);
    }

    /// `id`.
    pub fn attr_id(&mut self, value: &Id<'_>) {
        self.attr("id", value);
    }

    /// `name` de campo.
    pub fn attr_field_name(&mut self, value: &FieldName<'_>) {
        self.attr("name", value);
    }

    /// `width` em pixels (dimensão intrínseca: CLS 0).
    pub fn attr_width(&mut self, value: &crate::value::WidthPx) {
        self.attr("width", value);
    }

    /// `height` em pixels (dimensão intrínseca: CLS 0).
    pub fn attr_height(&mut self, value: &crate::value::HeightPx) {
        self.attr("height", value);
    }

    /// `datetime`.
    pub fn attr_timestamp(&mut self, value: &Timestamp) {
        self.attr("datetime", value);
    }

    /// Filho que implementa [`Render`].
    pub fn child(&mut self, value: &(impl Render + ?Sized)) {
        value.render_to(&mut self.buf);
    }

    /// Filho que é mensagem ([`Msg`]).
    pub fn msg<M: Msg>(&mut self, message: M) {
        escape::text(&mut self.buf, message.as_str());
    }
}
