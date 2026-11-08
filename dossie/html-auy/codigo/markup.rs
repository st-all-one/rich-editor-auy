//! `Markup` e o trait `Render`: o buffer único de saída.

use alloc::string::String;

/// HTML já renderizado, pronto para ir na resposta.
///
/// O campo é privado ao crate e **não existe** construtor a partir de `String`, `&str` ou
/// `Display`: um texto de fora não vira HTML. Os produtores são [`Render`] (primitivos do crate,
/// código confiável) e o macro `html!`. O bypass de escape não é proibido: é inexprimível.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Markup(pub(crate) String);

impl Markup {
    /// Renderiza qualquer [`Render`] num `Markup` novo.
    #[must_use]
    pub fn from_render(value: &(impl Render + ?Sized)) -> Self {
        let mut out = String::with_capacity(value.size_hint());
        value.render_to(&mut out);
        Self(out)
    }

    /// O HTML como texto.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consome o `Markup` e devolve o `String`, sem cópia.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }

    /// Tamanho do HTML em bytes.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// `true` quando o HTML está vazio.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Escreve HTML num `String` sem alocação intermediária.
///
/// Implementar `Render` é escrever **bytes confiáveis**: é o primitivo que permite compor sem
/// template em runtime. `Markup` implementa `Render`, então qualquer coisa que aceite `Markup`
/// aceita um primitivo.
pub trait Render {
    /// Estimativa de bytes para reservar o buffer (0 = não reserva).
    const SIZE_HINT: usize = 0;

    /// Escreve o HTML em `out`.
    fn render_to(&self, out: &mut String);

    /// Estimativa de bytes de `out` depois de [`Render::render_to`].
    fn size_hint(&self) -> usize {
        Self::SIZE_HINT
    }

    /// Renderiza num `Markup` novo.
    #[must_use]
    fn render(&self) -> Markup {
        let mut out = String::with_capacity(self.size_hint());
        self.render_to(&mut out);
        Markup(out)
    }
}

impl Render for Markup {
    fn render_to(&self, out: &mut String) {
        out.push_str(&self.0);
    }
}

impl<T: Render + ?Sized> Render for &T {
    fn render_to(&self, out: &mut String) {
        (*self).render_to(out);
    }
}
